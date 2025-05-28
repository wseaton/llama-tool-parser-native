use backend::parse_python;
use backend::parse_python_with_nom;
use backend::nom_parser::{NomParserState, parse_incremental};
use pyo3::prelude::*;
use pyo3::types::PyAny;
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

#[pyclass(name = "IncrementalParser")]
pub struct IncrementalParser {
    state: NomParserState,
}

#[pymethods]
impl IncrementalParser {
    #[new]
    fn new() -> Self {
        Self {
            state: NomParserState::new(),
        }
    }

    fn parse_chunk(&mut self, chunk: String) -> PyResult<Vec<PyObject>> {
        Python::with_gil(|py| {
            match parse_incremental(&mut self.state, &chunk) {
                Ok(function_calls) => Ok(pythonize(py, &function_calls)
                    .expect("Failed to pythonize")
                    .extract()
                    .expect("Failed to extract")),
                Err(err) => Err(PyErr::new::<pyo3::exceptions::PyValueError, _>(format!(
                    "Parse error: {:?}",
                    err
                ))),
            }
        })
    }

    fn reset(&mut self) {
        self.state.reset();
    }

    fn get_parsed_functions(&self) -> PyResult<Vec<PyObject>> {
        Python::with_gil(|py| {
            Ok(pythonize(py, &self.state.get_parsed_functions())
                .expect("Failed to pythonize")
                .extract()
                .expect("Failed to extract"))
        })
    }
}

#[pymodule]
fn llama_tool_parser_native(m: &Bound<'_, PyModule>) -> PyResult<()> {
    m.add_function(wrap_pyfunction!(wrapped_parse_python, m)?)?;
    m.add_class::<IncrementalParser>()?;
    Ok(())
}
