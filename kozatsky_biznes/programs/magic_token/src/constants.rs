/// Назва токена MagicToken
pub const MAGIC_TOKEN_NAME: &str = "MagicToken";

/// Символ токена MagicToken
pub const MAGIC_TOKEN_SYMBOL: &str = "MGT";

/// Розмір мінт-акаунту для базового Token-2022 без розширень (в байтах)
/// Base mint (82) + account type discriminator (1) + padding (2)
pub const BASIC_MINT_SIZE: usize = 85;
