use base64::{alphabet::STANDARD, engine::{DecodePaddingMode, GeneralPurpose, GeneralPurposeConfig}};

pub const BASE64_SANE: GeneralPurpose = GeneralPurpose::new(
	&STANDARD,
	GeneralPurposeConfig::new()
		.with_encode_padding(false)
		.with_decode_padding_mode(DecodePaddingMode::Indifferent),
);
