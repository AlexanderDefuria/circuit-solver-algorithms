use crate::validation::{Validation, ValidationResult};
use ndarray::Array2;
use std::fmt::{Debug, Formatter};
use std::ops::Add;
use std::rc::Rc;

pub trait EquationMember {
    fn equation_string(&self) -> String;
    fn value(&self) -> f64;
    fn is_zero(&self) -> bool {
        self.value().is_zero()
    }
    fn latex_string(&self) -> String {
        self.equation_string()
    }
}

#[derive(Debug, Clone)]
pub struct EquationRepr {
    string: String,
    latex: Option<String>,
    value: f64,
}

impl EquationMember for EquationRepr {
    fn equation_string(&self) -> String {
        self.string.clone()
    }
    fn value(&self) -> f64 {
        self.value
    }
    fn latex_string(&self) -> String {
        match &self.latex {
            Some(latex) => latex.clone(),
            None => self.equation_string(),
        }
    }
}

impl EquationRepr {
    pub fn new(string: String, value: f64) -> EquationRepr {
        EquationRepr {
            string,
            latex: None,
            value,
        }
    }

    pub fn new_with_latex(string: String, latex: String, value: f64) -> EquationRepr {
        EquationRepr {
            string,
            latex: Some(latex),
            value,
        }
    }
}

#[derive(Clone)]
pub(crate) enum MathOp {
    Multiply(Rc<dyn EquationMember>, Rc<dyn EquationMember>),
    Negate(Rc<dyn EquationMember>),
    Inverse(Rc<dyn EquationMember>),
    Divide(Rc<dyn EquationMember>, Rc<dyn EquationMember>),
    Sum(Vec<MathOp>),
    Collect(Rc<dyn EquationMember>),
    None(Rc<dyn EquationMember>),
    Unknown(EquationRepr),
}

impl num_traits::Zero for MathOp {
    fn zero() -> Self {
        MathOp::None(Rc::new(0.0))
    }

    fn is_zero(&self) -> bool {
        match self {
            MathOp::None(a) => a.is_zero(),
            _ => false,
        }
    }
}

impl Add for MathOp {
    type Output = Self;

    fn add(self, rhs: Self) -> Self::Output {
        match (self, rhs) {
            (MathOp::None(_), MathOp::None(_)) => MathOp::None(Rc::new(0.0)),
            (MathOp::None(_), b) => b,
            (a, MathOp::None(_)) => a,
            (a, b) => MathOp::Sum(vec![a, b]),
        }
    }
}

impl EquationMember for f64 {
    fn equation_string(&self) -> String {
        self.to_string()
    }
    fn value(&self) -> f64 {
        *self
    }
}

// TODO By changing the notation to latex we can use the same code for both
impl EquationMember for MathOp {
    fn equation_string(&self) -> String {
        match self {
            MathOp::Multiply(a, b) => {
                format!("{} * {}", a.equation_string(), b.equation_string())
            }
            MathOp::Negate(a) => {
                format!("-{}", a.equation_string())
            }
            MathOp::Inverse(a) => {
                format!("1/{}", a.equation_string())
            }
            MathOp::Divide(a, b) => {
                format!("{}/{}", a.equation_string(), b.equation_string())
            }
            MathOp::Sum(vec) => {
                let mut string = String::new();
                for (i, item) in vec.iter().enumerate() {
                    string.push_str(&item.equation_string());
                    if i != vec.len() - 1 {
                        string.push_str(" + ");
                    }
                }
                string
            }
            MathOp::Collect(a) => {
                let mut string = String::new();
                string.push_str("(");
                string.push_str(&a.equation_string());
                string.push_str(")");
                string
            }
            MathOp::None(a) => a.equation_string(),
            MathOp::Unknown(a) => a.equation_string(),
        }
    }

    fn value(&self) -> f64 {
        match self {
            MathOp::Multiply(a, b) => a.value() * b.value(),
            MathOp::Negate(a) => -a.value(),
            MathOp::Inverse(a) => 1.0 / a.value(),
            MathOp::Sum(vec) => {
                let mut sum = 0.0;
                for item in vec {
                    sum += item.value();
                }
                sum
            }
            MathOp::Divide(a, b) => a.value() / b.value(),
            MathOp::Collect(vec) => vec.value(),
            MathOp::None(a) => a.value(),
            MathOp::Unknown(a) => a.value(),
        }
    }

    fn latex_string(&self) -> String {
        match self {
            MathOp::Multiply(a, b) => {
                format!("{} \\cdot {}", a.latex_string(), b.latex_string())
            }
            MathOp::Negate(a) => {
                format!("-{}", a.latex_string())
            }
            MathOp::Inverse(a) => {
                format!("\\frac{{1}}{{{}}}", a.latex_string())
            }
            MathOp::Sum(vec) => {
                let mut string = String::new();
                for (i, item) in vec.iter().enumerate() {
                    string.push_str(&item.latex_string());
                    if i != vec.len() - 1 {
                        string.push_str(" + ");
                    }
                }
                string
            }
            MathOp::Divide(a, b) => {
                format!("\\frac{{{}}}{{{}}}", a.latex_string(), b.latex_string())
            }
            MathOp::Collect(a) => {
                let mut string = String::new();
                string.push_str("(");
                string.push_str(&a.latex_string());
                string.push_str(")");
                string
            }
            MathOp::None(a) => a.latex_string(),
            MathOp::Unknown(a) => a.latex_string(),
        }
    }
}

impl Validation for MathOp {
    fn validate(&self) -> ValidationResult {
        todo!()
    }
}

impl Debug for MathOp {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.equation_string())
    }
}

pub(crate) fn matrix_to_latex(matrix: Array2<MathOp>) -> String {
    let mut latex_a_matrix = String::new();
    latex_a_matrix.push_str("\\begin{bmatrix}");
    for row in matrix.genrows() {
        for (i, math) in row.iter().enumerate() {
            latex_a_matrix.push_str(&math.latex_string());
            if i != row.len() - 1 {
                latex_a_matrix.push_str(" & "); // Don't add & to last element
            }
        }
        latex_a_matrix.push_str("\\\\"); // End of row
    }
    latex_a_matrix.push_str("\\end{bmatrix}");
    latex_a_matrix
}

#[cfg(test)]
mod tests {
    use crate::component::Component::Resistor;
    use crate::math::{EquationMember, MathOp};
    use std::rc::Rc;

    #[test]
    fn test() {
        let a = Rc::new(MathOp::None(Rc::new(crate::elements::Element::new(
            Resistor,
            1.0,
            vec![1],
            vec![2],
        ))));
        let b = Rc::new(MathOp::None(Rc::new(crate::elements::Element {
            name: "R".to_string(),
            id: 1,
            value: 1.0,
            current: 0.0,
            voltage_drop: 0.0,
            class: Resistor,
            positive: vec![],
            negative: vec![],
        })));

        assert_eq!(a.equation_string(), "R0");
        assert_eq!(b.equation_string(), "R1");

        let neg_a = MathOp::Negate(a.clone());
        let inverse_b = MathOp::Inverse(b.clone());

        assert_eq!(neg_a.equation_string(), "-R0");
        assert_eq!(inverse_b.equation_string(), "1/R1");
        assert_eq!(neg_a.value(), -1.0);
        assert_eq!(inverse_b.value(), 1.0);

        let sum = Rc::new(MathOp::Sum(vec![neg_a, inverse_b]));
        assert_eq!(sum.value(), 0.0);
        assert_eq!(MathOp::Collect(sum).equation_string(), "(-R0 + 1/R1)");
        let set: Vec<MathOp> = vec![
            MathOp::None(Rc::new(1.0)),
            MathOp::None(Rc::new(2.0)),
            MathOp::None(Rc::new(3.0)),
        ];
        assert_eq!(MathOp::Sum(set).equation_string(), "1 + 2 + 3");
    }
}
