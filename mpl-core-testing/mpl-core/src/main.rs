use mpl_core::{
    instructions::{
        CreateCollectionV1Builder, CreateV1Builder, RemovePluginV1Builder, UpdateV1Builder,
    },
    types::{
        Attributes, DataState, FreezeDelegate, Plugin, PluginAuthority, PluginAuthorityPair,
        PluginType, UpdateDelegate,
    },
    Asset, Collection,
};
use solana_program_test::{
    tokio::{self, sync::Mutex},
    BanksClient, ProgramTest, ProgramTestContext,
};
use solana_sdk::{
    account::Account,
    msg,
    pubkey::Pubkey,
    signature::{Keypair, Signer},
    system_program,
    transaction::Transaction,
};

use spl_associated_token_account::get_associated_token_address;
use spl_token_2022::extension::metadata_pointer::MetadataPointer;
use spl_token_2022::extension::mint_close_authority::MintCloseAuthority;
use spl_token_2022::extension::{self, Extension};
use spl_token_2022::extension::{BaseStateWithExtensions, StateWithExtensions};
use spl_token_2022::state::Account as TokenAccount;
use spl_token_2022::state::Mint;
use std::str::FromStr;
pub mod util;
use std::sync::Arc;
use std_log::{debug, error, info, warn};
pub use util::*;

pub fn program_test() -> ProgramTest {
    let mut program_test = ProgramTest::default();

    // solana_logger::setup_with_default("solana_runtime=info");

    program_test.prefer_bpf(true);
    program_test.add_program("mpl_core", mpl_core::ID, None);
    program_test.add_program("spl_token_2022", spl_token_2022::ID, None);
    program_test.add_account_with_file_data(
        Pubkey::from_str("HJCZs1iZyf2goH3U61FksKtE8hW921XWCoRmxkpFKa9B").unwrap(),
        100,
        mpl_core::ID,
        "acc.json",
    );
    program_test.add_account_with_file_data(
        Pubkey::from_str("771QCuckXqoxkhUcsE7xK6ieJVCQRhyB2wwYsDcSynHu").unwrap(),
        100,
        mpl_core::ID,
        "acc1.json",
    );
    program_test
}

pub async fn get_account(context: Arc<Mutex<ProgramTestContext>>, pubkey: &Pubkey) -> Account {
    let context = &mut *context.lock().await;
    context
        .banks_client
        .get_account(*pubkey)
        .await
        .unwrap()
        .expect("account not found")
}

fn keypair_clone(kp: &Keypair) -> Keypair {
    Keypair::from_bytes(&kp.to_bytes()).expect("failed to copy keypair")
}

#[derive(Debug)]
pub struct CreateAssetHelperArgs<'a> {
    pub owner: Option<Pubkey>,
    pub payer: Option<&'a Keypair>,
    pub asset: &'a Keypair,
    pub data_state: Option<DataState>,
    pub name: Option<String>,
    pub uri: Option<String>,
    pub authority: Option<Pubkey>,
    pub update_authority: Option<Pubkey>,
    pub collection: Option<Pubkey>,
    // TODO use PluginList type here
    pub plugins: Vec<PluginAuthorityPair>,
}

#[tokio::main]
async fn main() {
    let mut context = program_test().start_with_context().await;
    let random_keypair = Keypair::new();
    // let ctx = Arc::new(Mutex::new(context));
    let payer = context.payer;
    let input = CreateAssetHelperArgs {
        owner: Some(Keypair::new().pubkey()),
        payer: None,
        asset: &Keypair::new(),
        data_state: None,
        name: Some("None".to_string()),
        uri: Some("None".to_string()),
        authority: Some(payer.pubkey()),
        update_authority: Some(payer.pubkey()),
        collection: None,
        plugins: vec![
            PluginAuthorityPair {
                plugin: Plugin::Attributes(Attributes {
                    attribute_list: vec![],
                }),
                authority: Some(PluginAuthority::Address {
                    address: payer.pubkey(),
                }),
            },
            PluginAuthorityPair {
                plugin: Plugin::UpdateDelegate(UpdateDelegate {
                    additional_delegates: vec![],
                }),
                authority: Some(PluginAuthority::Address {
                    address: payer.pubkey(),
                }),
            },
            PluginAuthorityPair {
                plugin: Plugin::FreezeDelegate(FreezeDelegate { frozen: false }),
                authority: Some(PluginAuthority::Address {
                    address: payer.pubkey(),
                }),
            },
        ],
    };

    const DEFAULT_ASSET_NAME: &str = "default_asset_name";
    const DEFAULT_ASSET_URI: &str = "default_asset_uri";

    let collection = context
        .banks_client
        .get_account(Pubkey::from_str("HJCZs1iZyf2goH3U61FksKtE8hW921XWCoRmxkpFKa9B").unwrap())
        .await
        .unwrap()
        .expect("collection not found");
    let col = mpl_core::Collection::from_bytes(&collection.data).unwrap();
    println!("Collection: {:?}", col);
    let collection = context
        .banks_client
        .get_account(Pubkey::from_str("771QCuckXqoxkhUcsE7xK6ieJVCQRhyB2wwYsDcSynHu").unwrap())
        .await
        .unwrap()
        .expect("collection not found");
    let col = mpl_core::Collection::from_bytes(&collection.data).unwrap();
    println!("Collection: {:?}", col);
    return;
    // let core2 = CreateCollectionV1Builder::new()
    //     .collection(input.collection)
    //     .update_authority(Some(payer.pubkey()))
    //     .payer(payer.pubkey())
    //     .name("collection_name".to_string())
    //     .symbol("collection_symbol".to_string())
    //     .uri("collection_uri".to_string())
    //     .instruction();

    let core_1 = CreateV1Builder::new()
        .asset(input.asset.pubkey())
        .collection(input.collection)
        .authority(input.authority)
        .payer(payer.pubkey())
        .owner(Some(input.owner.unwrap_or(payer.pubkey())))
        .update_authority(input.update_authority)
        .system_program(system_program::ID)
        .data_state(input.data_state.unwrap_or(DataState::AccountState))
        .name(input.name.unwrap_or(DEFAULT_ASSET_NAME.to_owned()))
        .uri(input.uri.unwrap_or(DEFAULT_ASSET_URI.to_owned()))
        .plugins(input.plugins)
        .instruction();

    // let core_2 = UpdateV1Builder::new()
    //     .asset(input.asset.pubkey())
    //     .collection(input.collection)
    //     .authority(Some(random_keypair.pubkey()))
    //     .payer(payer.pubkey())
    //     .new_name("new_name".to_string())
    //     .new_uri("new_uri".to_string())
    //     .instruction();

    let core_3 = RemovePluginV1Builder::new()
        .asset(input.asset.pubkey())
        .collection(input.collection)
        .authority(Some(payer.pubkey()))
        .payer(payer.pubkey())
        .plugin_type(PluginType::UpdateDelegate)
        .instruction();

    // let core = CreateV1Builder::new()
    //     .asset(input.asset.pubkey())
    //     .collection(input.collection)
    //     .authority(input.authority)
    //     .payer(payer.pubkey())
    //     .owner(Some(input.owner.unwrap_or(payer.pubkey())))
    //     .update_authority(input.update_authority)
    //     .system_program(system_program::ID)
    //     .data_state(input.data_state.unwrap_or(DataState::AccountState))
    //     .name(input.name.unwrap_or(DEFAULT_ASSET_NAME.to_owned()))
    //     .uri(input.uri.unwrap_or(DEFAULT_ASSET_URI.to_owned()))
    //     .plugins(input.plugins)
    //     .instruction();

    let mut signers = vec![input.asset, &payer];
    if let Some(payer) = input.payer {
        signers.push(payer);
    }

    let tx = Transaction::new_signed_with_payer(
        &[core_1, core_3],
        Some(&payer.pubkey()),
        signers.as_slice(),
        context.last_blockhash,
    );

    let res = context.banks_client.process_transaction(tx).await;
    match res {
        Ok(res) => {
            info!("Transaction executed. Result: {:?}", res);
        }
        Err(err) => {
            error!("Transaction failed. Error: {:?}", err);
        }
    }

    let account = context
        .banks_client
        .get_account(input.asset.pubkey())
        .await
        .unwrap()
        .expect("account not found");
    // let asset = mpl_core::Asset::deserialize_asset(&account.data).unwrap();
    // println!("Asset: {:?}", asset);
}
