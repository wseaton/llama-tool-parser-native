use backend::parse_python;
use backend::parse_python_with_nom;
use pyo3::prelude::*;
use pythonize::pythonize;

#[pyfunction(name = "parse_tools")]
pub fn wrapped_parse_python(
    py: Python<'_>,
    source: String,
    engine: String,
) -> PyResult<Bound<'_, PyAny>> {
    let function_calls = match engine.as_str() {
        "nom" => match parse_python_with_nom(&source) {
            Ok(function_calls) => Ok(function_calls),
            Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Parse error: {:?}",
                err
            ))),
        },
        "logos" => match parse_python(&source) {
            Ok(function_calls) => Ok(function_calls),
            Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                "Parse error: {:?}",
                err
            ))),
        },
        _ => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Unsupported engine: {}",
            engine
        ))),
    };

    if let Ok(function_calls) = function_calls {
        Ok(pythonize(py, &function_calls)
            .expect("Failed to pythonize")
            .to_owned())
    } else {
        Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
            "Parse error: {:?}",
            function_calls
        )))
    }
}

#[pymodule]
fn llama_tool_parser_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(wrapped_parse_python, m)?)
}
