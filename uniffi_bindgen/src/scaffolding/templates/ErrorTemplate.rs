{#
// For each error declared in the UDL, we assume the caller has provided a corresponding
// rust `enum`. We provide the traits for sending it across the FFI, which will fail to
// compile if the provided struct has a different shape to the one declared in the UDL.
//
// We define a unit-struct to implement the trait to sidestep Rust's orphan rule (ADR-0006). It's
// public so other crates can refer to it via an `[External='crate'] typedef`
#}

#[::uniffi::ffi_converter_error(crate::UniFfiTag)]
{%- if e.is_flat() %}
#[uniffi(flat_error{% if ci.should_generate_error_read(e) %},with_try_read{% endif %})]
{%- endif %}
enum r#{{ e.name() }} {
    {%- for variant in e.variants() %}
    r#{{ variant.name() }} {
        {%- for field in variant.fields() %}
        r#{{ field.name() }}: {{ field.type_()|type_rs }},
        {%- endfor %}
    },
    {%- endfor %}
}
