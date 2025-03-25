use crate::model::connection::Connection;

#[derive(Clone, Debug)]
pub struct ProcessContext<'a> {
    pub(crate) current_step: usize,
    pub(crate) current_time: f64,
    pub(crate) inputs: Vec<&'a Connection>,
    pub(crate) outputs: Vec<&'a Connection>,
}

impl<'a> Default for ProcessContext<'a> {
    fn default() -> Self {
        Self {
            current_step: 0,
            current_time: 0.0,
            inputs: vec![],
            outputs: vec![],
        }
    }
}

impl<'a> ProcessContext<'a> {
    pub fn new(
        current_step: usize,
        current_time: f64,
        inputs: Vec<&'a Connection>,
        outputs: Vec<&'a Connection>,
    ) -> Self {
        Self {
            current_step,
            current_time,
            inputs,
            outputs,
        }
    }

    pub fn current_step(&self) -> usize {
        self.current_step
    }

    pub fn current_time(&self) -> f64 {
        self.current_time
    }

    pub fn inputs_for_port(&self, port: Option<&str>) -> impl Iterator<Item = &Connection> {
        let port_str = port.map(String::from);
        self.inputs
            .iter()
            .copied()
            .filter(move |conn| conn.target_port == port_str)
    }

    pub fn outputs_for_port(&self, port: Option<&str>) -> impl Iterator<Item = &Connection> {
        let port_str = port.map(String::from);
        self.outputs
            .iter()
            .copied()
            .filter(move |conn| conn.source_port == port_str)
    }
}
