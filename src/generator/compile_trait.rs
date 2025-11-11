use super::generator::Generator;

pub trait Generate {
    fn generate(&self, generator: &mut Generator);
}
