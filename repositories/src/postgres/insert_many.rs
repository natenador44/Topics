use std::{fmt::Write, marker::PhantomData};

#[cfg(test)]
use dyn_eq::DynEq;
use itertools::Itertools;
use tokio_postgres::types::ToSql;

#[cfg(not(test))]
pub trait InsertManyValue: ToSql + Send + Sync + 'static {}

#[cfg(not(test))]
impl<T> InsertManyValue for T where T: ToSql + Send + Sync + 'static {}

#[cfg(test)]
pub trait InsertManyValue: ToSql + Send + Sync + DynEq + 'static {}

#[cfg(test)]
impl<T> InsertManyValue for T where T: ToSql + Send + Sync + DynEq + 'static {}

pub struct InsertMany {
    pub query: String,
    params: Vec<Box<dyn InsertManyValue>>,
}

impl InsertMany {
    pub fn params(&self) -> Vec<&(dyn ToSql + Sync)> {
        self.params
            .iter()
            .map(|p| &**p as &(dyn ToSql + Sync))
            .collect()
    }
}

macro_rules! value_set {
    ($($val:expr => $t:ty),+) => {
        crate::postgres::insert_many::ValueSet::<_, ($($t,)+)>::new([$(crate::postgres::insert_many::Value::from($val)),+])
    };
}

pub(crate) use value_set;

pub struct Value(Box<dyn InsertManyValue>);

impl<T> From<T> for Value
where
    T: InsertManyValue,
{
    fn from(value: T) -> Self {
        Self(Box::new(value))
    }
}

pub struct ValueSet<const N: usize, T> {
    values: [Value; N],
    _phantom: PhantomData<T>,
}

impl<const N: usize, T> ValueSet<N, T> {
    pub fn new(values: [Value; N]) -> Self {
        Self {
            values,
            _phantom: PhantomData,
        }
    }
}

pub struct InsertManyBuilder<const COLS: usize, T> {
    table: &'static str,
    col_names: [&'static str; COLS],
    value_sets: Vec<ValueSet<COLS, T>>,
    returning: Option<&'static [&'static str]>,
}

impl<const COLS: usize, T> InsertManyBuilder<COLS, T> {
    pub fn new(
        table: &'static str,
        col_names: [&'static str; COLS],
        starting_set: ValueSet<COLS, T>,
    ) -> Self {
        Self {
            table,
            col_names,
            value_sets: vec![starting_set],
            returning: None,
        }
    }

    pub fn add_value_set(&mut self, value: ValueSet<COLS, T>) -> &mut Self {
        self.value_sets.push(value);
        self
    }

    pub fn returning(&mut self, cols: &'static [&'static str]) -> &mut Self {
        self.returning = Some(cols);
        self
    }

    pub fn build(self) -> InsertMany {
        let mut query = format!("INSERT INTO {} ", self.table);

        let value_name_segments = Itertools::intersperse(self.col_names.iter().map(|s| *s), ",");

        query.push('(');
        for segment in value_name_segments {
            query.push_str(segment);
        }
        query.push(')');
        query.push_str(" VALUES ");

        let value_set_count = self.value_sets.len();

        let mut params = Vec::with_capacity(value_set_count * COLS);
        let mut param_count = 0;
        for (i, value_set) in self.value_sets.into_iter().enumerate() {
            query.push('(');
            for j in 1..=value_set.values.len() {
                write!(&mut query, "${}", param_count + j).expect("write to string");
                if j < value_set.values.len() {
                    query.push(',');
                }
            }
            query.push(')');

            if i < value_set_count - 1 {
                query.push(',');
            }

            param_count += value_set.values.len();
            params.extend(value_set.values.into_iter().map(|v| v.0));
        }

        if let Some(returning) = self.returning {
            query += " RETURNING ";

            for segment in Itertools::intersperse(returning.into_iter().map(|s| *s), ",") {
                query += segment;
            }
        }

        InsertMany { query, params }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use dyn_eq::DynEq;

    dyn_eq::eq_trait_object!(InsertManyValue);

    const TEST_TABLE_NAME: &str = "test";

    macro_rules! expected_params {
        ($($e:expr),*) => {
            vec![$(Box::new($e) as Box<dyn InsertManyValue>),*]
        };
    }

    #[test]
    fn build_single_value_set_with_single_value() {
        let insert_many =
            InsertManyBuilder::new(TEST_TABLE_NAME, ["col1"], value_set!(42 => i32)).build();

        assert_eq!("INSERT INTO test (col1) VALUES ($1)", &insert_many.query);
        let expected_params = expected_params!(42);
        assert_eq!(expected_params, insert_many.params);
    }

    #[test]
    fn build_single_value_set_with_multiple_values() {
        let insert_many = InsertManyBuilder::new(
            TEST_TABLE_NAME,
            ["col1", "col2", "col3"],
            value_set![42 => i32, "hello" => &'static str, true => bool],
        )
        .build();

        assert_eq!(
            "INSERT INTO test (col1,col2,col3) VALUES ($1,$2,$3)",
            &insert_many.query
        );

        let expected_params = expected_params!(42, "hello", true);
        assert_eq!(expected_params, insert_many.params);
    }

    #[test]
    fn build_multiple_value_sets_with_single_value() {
        let mut builder = InsertManyBuilder::new(TEST_TABLE_NAME, ["col1"], value_set![42 => i32]);

        for i in 0..5 {
            builder.add_value_set(value_set![i => i32]);
        }

        let insert_many = builder.build();

        assert_eq!(
            "INSERT INTO test (col1) VALUES ($1),($2),($3),($4),($5),($6)",
            insert_many.query,
        );

        let expected_params = expected_params!(42, 0, 1, 2, 3, 4);
        assert_eq!(expected_params, insert_many.params);
    }

    #[test]
    fn build_multiple_value_sets_with_multiple_values_per() {
        let mut builder = InsertManyBuilder::new(
            TEST_TABLE_NAME,
            ["col1", "col2", "col3"],
            value_set![42 => i32, true => bool, "hello" => &'static str],
        );

        builder.add_value_set(value_set![100 => i32, false => bool, "test" => &'static str]);
        builder.add_value_set(value_set![2 => i32, false => bool, "world" => &'static str]);

        let insert_many = builder.build();

        assert_eq!(
            "INSERT INTO test (col1,col2,col3) VALUES ($1,$2,$3),($4,$5,$6),($7,$8,$9)",
            insert_many.query,
        );

        let expected_params =
            expected_params!(42, true, "hello", 100, false, "test", 2, false, "world");
        assert_eq!(expected_params, insert_many.params);
    }

    #[test]
    fn build_with_returning() {
        let mut builder = InsertManyBuilder::new(TEST_TABLE_NAME, ["col1"], value_set!(42 => i32));
        builder.returning(&["id", "name", "description"]);
        let insert_many = builder.build();

        assert_eq!(
            "INSERT INTO test (col1) VALUES ($1) RETURNING id,name,description",
            insert_many.query,
        )
    }
}
