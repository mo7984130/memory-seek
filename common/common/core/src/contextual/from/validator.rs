use validator::{ValidationErrors, ValidationErrorsKind};

pub fn format_validation_error_msg(errors: &ValidationErrors) -> String {
    errors
        .errors()
        .iter()
        .filter_map(|(field, kind)| match kind {
            ValidationErrorsKind::Field(errors) => Some(
                errors
                    .iter()
                    .map(|err| {
                        err.message
                            .as_deref()
                            .map(str::to_owned)
                            .unwrap_or_else(|| format!("字段 '{field}' 校验失败"))
                    })
                    .collect::<Vec<_>>(),
            ),
            _ => None,
        })
        .flatten()
        .collect::<Vec<_>>()
        .join("; ")
}
