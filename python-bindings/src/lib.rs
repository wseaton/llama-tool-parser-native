use backend::parse_python;
use pyo3::prelude::*;
use pythonize::pythonize;

#[pyfunction(name = "parse_tools")]
pub fn wrapped_parse_python(py: Python, source: String) -> PyResult<Bound<'_, PyAny>> {
    match parse_python(&source) {
        Ok(function_calls) => Ok(pythonize(py, &function_calls).expect("Failed to pythonize")),
        Err((msg, span)) => {
            let error_message = format!("Error at position {}-{}: {}", span.start, span.end, msg);
            Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(
                error_message,
            ))
        }
    }
}

#[pymodule]
fn llama_tool_parser_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(wrapped_parse_python, m)?)
}
