use crate::resources::ResourceError;

#[derive(Copy, Clone, PartialEq, Eq, Debug, Hash)]
pub enum SerialID {
    Usart1,
    Usart2,
    Usart3,
    Usart4,
}
impl SerialID {
    #[inline]
    pub fn from_str(id: &str) -> Result<Self, ResourceError> {
        match id {
            "usart1" | "Usart1" | "USART1" => Ok(SerialID::Usart1),
            "usart2" | "Usart2" | "USART2" => Ok(SerialID::Usart2),
            "usart3" | "Usart3" | "USART3" => Ok(SerialID::Usart3),
            "usart4" | "Usart4" | "USART4" => Ok(SerialID::Usart4),
            _ => Err(ResourceError::ParseError),
        }
    }
}
