use std::ffi::{c_char, c_int};

#[link(name = "CE", kind = "raw-dylib")]
extern "C" {
    /// Initializes the CELUA DLL.
    ///
    /// Arguments:
    /// - name: The name of the lua server to connect to.
    ///
    /// Returns a boolean indicating success.
    pub fn CELUA_Initialize(name: *const c_char) -> c_int;

    /// Executes lua code on the main CE UI thread.
    ///
    /// Arguments:
    /// - luacode: The lua code to execute.
    /// - param: An integer parameter to the function. Called "parameter" in the lua code's context.
    ///
    /// Returns the return value of the function, if integral. Undefined otherwise.
    pub fn CELUA_ExecuteFunction(luacode: *const c_char, parameter: usize) -> usize;

    /// Executes lua code in the lua server (not waiting for the UI thread).
    ///
    /// Arguments:
    /// - luacode: The lua code to execute.
    /// - param: An integer parameter to the function. Called "parameter" in the lua code's context.
    ///
    /// Returns the return value of the function, if integral. Undefined otherwise.
    pub fn CELUA_ExecuteFunctionAsync(luacode: *const c_char, parameter: usize) -> usize;

    /// Gets a reference ID which can be used to call an existing lua function via [`CELUA_ExecuteFunctionByReference`].
    ///
    /// Arguments:
    /// - function_name: The name of the function to obtain a reference to.
    ///
    /// Returns the function's unique integer ID.
    pub fn CELUA_GetFunctionReferenceFromName(function_name: *const c_char) -> c_int;

    /// Executes the function specified by reference id.
    ///
    /// Arguments:
    /// -  ref_id: ID of the function to execute, obtained with CELUA_GetFunctionReferenceFromName.
    /// -  param_count: Number of parameters to be passed to the function.
    /// -  parameters: An array of integer parameters which will be passed to the function.
    /// -  is_async: If true, the code will run in a seperate thread instead of the main thread.
    ///
    /// Returns the return value of the function, if integral. Undefined otherwise.
    pub fn CELUA_ExecuteFunctionByReference(
        ref_id: c_int,
        param_count: usize,
        parameters: *const usize,
        is_async: c_int,
    ) -> usize;
}
