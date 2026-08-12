mod parsing {
    #![allow(clippy::result_large_err)]
    // ^ Figment Jail test wrapper returns large error types. Not much we can do
    // about that, so ignore it for all the parsing tests that use it.

    mod challenges;
    mod config;
}

mod init;
