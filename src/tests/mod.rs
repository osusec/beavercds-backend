mod parsing {
    #![allow(clippy::result_large_err)]
    // ^ Figment jails in these tests return a large Error result, not much we
    // can do to fix that
    mod challenges;
    mod config;
}

mod init;
