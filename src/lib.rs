#[allow(dead_code)]
pub mod common;

#[allow(dead_code)]
pub mod transaction;

#[allow(dead_code)]
pub mod accounts;

#[allow(dead_code)]
pub mod writeprogram;

#[allow(dead_code)]
pub mod getclock;

#[allow(dead_code)]
pub mod create_data_account;

#[allow(dead_code)]
pub mod token {
    pub mod confidential_transfer;
    pub mod create_mint_account;
    pub mod create_token_account;
    pub mod mint_token;
    pub mod token_transfer;

    pub mod token_op;

    pub mod comm;
}

#[allow(dead_code)]
pub mod lite_svm;

#[allow(dead_code)]
pub mod bin_code_demo;

#[allow(dead_code)]
pub mod client {
    pub mod program_client;
}
