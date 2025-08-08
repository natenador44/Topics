use error_set::error_set;

error_set! {
    Error = {
        InitLogging,
        InitPort,
        InitServe
    };
}
