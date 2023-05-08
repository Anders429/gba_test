use bincode::config;

/// Configuration for bincode encoding.
///
/// This definition ensures that the same configuration can be used across all code.
#[cfg_attr(doc_cfg, doc(cfg(feature = "bincode")))]
pub const BINCODE_CONFIG: config::Configuration<
    config::LittleEndian,
    config::Fixint,
    config::NoLimit,
> = config::standard().with_fixed_int_encoding();
