// use crate::{get_account, TokenManager};
// use mpl_token_metadata::{
//     accounts::{
//         EditionMarker, EditionMarkerV2, HolderDelegateRecord, MasterEdition, Metadata, TokenRecord,
//     },
//     instructions::{
//         CreateMetadataAccountV3Builder, CreateV1Builder, DelegatePrintDelegateV1Builder, Mint,
//         MintV1Builder, PrintV2, PrintV2Builder, UpdateAsUpdateAuthorityV2,
//         UpdateAsUpdateAuthorityV2Builder, UpdateV1Builder, VerifyCollectionV1Builder,
//     },
//     types::{Collection, HolderDelegateRole, Key, PrintSupply, TokenDelegateRole, TokenStandard},
//     EDITION_MARKER_BIT_SIZE,
// };
// use serde::de::Error as SerdeError;
// use serde::{Deserialize, Deserializer, Serialize, Serializer};
// use serde_bytes::{ByteBuf as SerdeByteBuf, Bytes as SerdeBytes};
// use solana_program::pubkey::Pubkey;
// use solana_program_test::tokio::sync::Mutex;
// use solana_program_test::{BanksClientError, ProgramTestContext};
// use solana_sdk::{
//     compute_budget::ComputeBudgetInstruction, signature::Keypair, signer::Signer,
//     transaction::Transaction,
// };
// use spl_token_2022::extension::ExtensionType;
// use std::{
//     ops::{Deref, DerefMut},
//     sync::Arc,
// };

// #[derive(Debug)]
// pub struct MyKeypair(Keypair);

// impl Serialize for MyKeypair {
//     fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
//     where
//         S: Serializer,
//     {
//         let bytes = &self.0.to_bytes()[..];
//         SerdeBytes::new(bytes).serialize(serializer)
//     }
// }

// impl Deref for MyKeypair {
//     type Target = Keypair;

//     fn deref(&self) -> &Self::Target {
//         &self.0
//     }
// }

// impl DerefMut for MyKeypair {
//     fn deref_mut(&mut self) -> &mut Self::Target {
//         &mut self.0
//     }
// }

// impl MyKeypair {
//     // Define a new method for MyKeypair that internally calls Keypair's new method
//     pub fn new() -> Self {
//         MyKeypair(Keypair::new())
//     }
// }

// #[derive(Debug, Serialize)]
// pub struct MintManager {
//     pub mint: MyKeypair,
//     pub owner: MyKeypair,
//     pub token_account: Pubkey,
//     pub metadata: Pubkey,
//     pub master_edition: Pubkey,
//     pub update_authority: MyKeypair,
// }

// impl Default for MintManager {
//     fn default() -> Self {
//         let mint = MyKeypair::new();
//         let mint_public_key = mint.pubkey();
//         let owner = MyKeypair::new();
//         Self {
//             mint: mint,
//             owner: owner,
//             token_account: Pubkey::default(),
//             metadata: Metadata::find_pda(&mint_public_key).0,
//             master_edition: MasterEdition::find_pda(&mint_public_key).0,
//             update_authority: MyKeypair::new(),
//         }
//     }
// }

// impl MintManager {
//     #[allow(clippy::too_many_arguments)]
//     pub async fn create(
//         &mut self,
//         context: Arc<Mutex<ProgramTestContext>>,
//         name: String,
//         uri: String,
//         token_standard: TokenStandard,
//         spl_token_program: Pubkey,
//         print_supply: u64,
//         collection_mint: Option<Pubkey>,
//     ) -> Result<(), BanksClientError> {
//         // Move these to default?
//         let mut context = context.lock().await;
//         self.token_account =
//             spl_associated_token_account::get_associated_token_address_with_program_id(
//                 &self.owner.pubkey(),
//                 &self.mint.pubkey(),
//                 &spl_token_program,
//             );

//         let mut create_builder = CreateV1Builder::new();

//         let create_ix = create_builder
//             .metadata(self.metadata)
//             .master_edition(Some(self.master_edition))
//             .mint(self.mint.pubkey(), true)
//             .authority(self.update_authority.pubkey())
//             .payer(context.payer.pubkey())
//             .update_authority(self.update_authority.pubkey(), true)
//             .is_mutable(true)
//             .primary_sale_happened(false)
//             .seller_fee_basis_points(500)
//             .print_supply(PrintSupply::Limited(print_supply))
//             .name(name)
//             .uri(uri)
//             .token_standard(token_standard)
//             .spl_token_program(Some(spl_token_program));

//         let create_ix = match collection_mint {
//             Some(collection_mint) => {
//                 let collection = Collection {
//                     verified: false,
//                     key: collection_mint,
//                 };
//                 create_ix.collection(collection)
//             }
//             None => create_ix,
//         };

//         let create_ix = create_ix.instruction();

//         let tx = Transaction::new_signed_with_payer(
//             &[create_ix],
//             Some(&context.payer.pubkey()),
//             &[&context.payer, &self.update_authority, &self.mint],
//             context.last_blockhash,
//         );
//         context.banks_client.process_transaction(tx).await
//     }

//     pub async fn mint(
//         &mut self,
//         context: Arc<Mutex<ProgramTestContext>>,
//         spl_token_program: Pubkey,
//     ) -> Result<(), BanksClientError> {
//         let mut context = context.lock().await;
//         let token_record = TokenRecord::find_pda(&self.mint.pubkey(), &self.token_account).0;

//         let mint_ix = MintV1Builder::new()
//             .token(self.token_account)
//             .token_owner(Some(self.owner.pubkey()))
//             .metadata(self.metadata)
//             .master_edition(Some(self.master_edition))
//             .token_record(Some(token_record))
//             .mint(self.mint.pubkey())
//             .authority(self.update_authority.pubkey())
//             .payer(context.payer.pubkey())
//             .amount(1)
//             .spl_token_program(spl_token_program)
//             .instruction();

//         let compute_unit_ix = ComputeBudgetInstruction::set_compute_unit_limit(1000000);

//         let tx = Transaction::new_signed_with_payer(
//             &[compute_unit_ix, mint_ix],
//             Some(&context.payer.pubkey()),
//             &[&context.payer, &self.update_authority],
//             context.last_blockhash,
//         );
//         context.banks_client.process_transaction(tx).await
//     }

//     #[allow(clippy::too_many_arguments)]
//     pub async fn update(
//         &mut self,
//         context: Arc<Mutex<ProgramTestContext>>,
//         token_standard: TokenStandard,
//     ) -> Result<(), BanksClientError> {
//         // Move these to default?
//         let mut context = context.lock().await;
//         let mut update_builder = UpdateAsUpdateAuthorityV2Builder::new();
//         let update_ix = update_builder
//             .authority(self.update_authority.pubkey())
//             .token(Some(self.token_account))
//             .mint(self.mint.pubkey())
//             .metadata(self.metadata)
//             .authority(self.update_authority.pubkey())
//             .payer(context.payer.pubkey())
//             .is_mutable(true)
//             .primary_sale_happened(false)
//             .token_standard(token_standard)
//             .instruction();

//         let tx = Transaction::new_signed_with_payer(
//             &[update_ix],
//             Some(&context.payer.pubkey()),
//             &[&context.payer, &self.update_authority],
//             context.last_blockhash,
//         );
//         context.banks_client.process_transaction(tx).await
//     }

//     #[allow(clippy::too_many_arguments)]
//     pub async fn delegate(
//         &mut self,
//         context: Arc<Mutex<ProgramTestContext>>,
//         delegate: &Pubkey,
//         spl_token_program: Pubkey,
//     ) -> Result<(), BanksClientError> {
//         let mut context = context.lock().await;
//         let token_record = TokenRecord::find_pda(&self.mint.pubkey(), &self.token_account).0;
//         let delegate_record = HolderDelegateRecord::find_pda(
//             &self.mint.pubkey(),
//             HolderDelegateRole::PrintDelegate,
//             &self.owner.pubkey(),
//             delegate,
//         )
//         .0;

//         let delegate_ix = DelegatePrintDelegateV1Builder::new()
//             .delegate_record(Some(delegate_record))
//             .delegate(*delegate)
//             .metadata(self.metadata)
//             .master_edition(Some(self.master_edition))
//             .token_record(Some(token_record))
//             .mint(self.mint.pubkey())
//             .token(Some(self.token_account))
//             .authority(self.owner.pubkey())
//             .payer(context.payer.pubkey())
//             .spl_token_program(Some(spl_token_program))
//             .instruction();

//         let tx = Transaction::new_signed_with_payer(
//             &[delegate_ix],
//             Some(&context.payer.pubkey()),
//             &[&context.payer, &self.owner],
//             context.last_blockhash,
//         );
//         context.banks_client.process_transaction(tx).await
//     }

//     pub async fn verify(
//         &mut self,
//         context: Arc<Mutex<ProgramTestContext>>,
//         collection_mint: &mut MintManager,
//         spl_token_program: Pubkey,
//     ) -> Result<(), BanksClientError> {
//         let mut context = context.lock().await;
//         let verify_ix = VerifyCollectionV1Builder::new()
//             .authority(collection_mint.update_authority.pubkey())
//             .metadata(self.metadata)
//             .collection_mint(collection_mint.mint.pubkey())
//             .collection_metadata(Some(collection_mint.metadata))
//             .collection_master_edition(Some(collection_mint.master_edition))
//             .instruction();

//         let tx = Transaction::new_signed_with_payer(
//             &[verify_ix],
//             Some(&context.payer.pubkey()),
//             &[&context.payer, &collection_mint.update_authority],
//             context.last_blockhash,
//         );
//         context.banks_client.process_transaction(tx).await
//     }

//     //
//     pub async fn print(
//         &mut self,
//         context: Arc<Mutex<ProgramTestContext>>,
//         print_info: &mut MintManager,
//         delegate: &Keypair,
//         spl_token_program: Pubkey,
//         edition_number: u64,
//         token_standard: TokenStandard,
//     ) -> Result<(), BanksClientError> {
//         let mut context = context.lock().await;
//         print_info.token_account =
//             spl_associated_token_account::get_associated_token_address_with_program_id(
//                 &print_info.owner.pubkey(),
//                 &print_info.mint.pubkey(),
//                 &spl_token::ID,
//             );
//         let edition_token_record =
//             TokenRecord::find_pda(&print_info.mint.pubkey(), &print_info.token_account).0;
//         let edition_marker_pda = match token_standard {
//             TokenStandard::NonFungible => {
//                 let edition_marker = edition_number.checked_div(EDITION_MARKER_BIT_SIZE).unwrap();
//                 EditionMarker::find_pda(&self.mint.pubkey(), &edition_marker.to_string()).0
//             }
//             TokenStandard::ProgrammableNonFungible => {
//                 EditionMarkerV2::find_pda(&self.mint.pubkey()).0
//             }
//             _ => {
//                 return Err(BanksClientError::ClientError(("invalid token standard")));
//             }
//         };
//         let holder_delegate_record = HolderDelegateRecord::find_pda(
//             &self.mint.pubkey(),
//             HolderDelegateRole::PrintDelegate,
//             &self.owner.pubkey(),
//             &delegate.pubkey(),
//         )
//         .0;

//         let print_tx = PrintV2Builder::new()
//             .edition_metadata(print_info.metadata)
//             .edition(print_info.master_edition)
//             .edition_mint(print_info.mint.pubkey(), true)
//             .edition_mint_authority(context.payer.pubkey())
//             .edition_token_account(print_info.token_account)
//             .edition_token_account_owner(print_info.owner.pubkey())
//             .edition_token_record(Some(edition_token_record))
//             .edition_marker_pda(edition_marker_pda)
//             .master_edition(self.master_edition)
//             .master_metadata(self.metadata)
//             .update_authority(self.update_authority.pubkey())
//             .master_token_account(self.token_account)
//             .master_token_account_owner(self.owner.pubkey(), false)
//             .holder_delegate_record(Some(holder_delegate_record))
//             .delegate(Some(delegate.pubkey()))
//             .spl_token_program(spl_token::ID)
//             .edition_number(edition_number)
//             .payer(context.payer.pubkey())
//             .instruction();

//         let compute_unit_ix = ComputeBudgetInstruction::set_compute_unit_limit(1000000);
//         let tx = Transaction::new_signed_with_payer(
//             &[compute_unit_ix, print_tx],
//             Some(&context.payer.pubkey()),
//             &[&context.payer, &print_info.mint, &delegate],
//             context.last_blockhash,
//         );
//         context.banks_client.process_transaction(tx).await
//     }

//     pub async fn print_metadata(&mut self, context: Arc<Mutex<ProgramTestContext>>) {
//         let metadata_account = get_account(context, &self.metadata).await;
//         let parsed_account = Metadata::from_bytes(&metadata_account.data).unwrap();
//         println!("{}", "-".repeat(80));
//         println!("{}", parsed_account);
//         println!("{}", "-".repeat(80));
//     }

//     // #[allow(clippy::too_many_arguments)]
//     // pub async fn create_and_mint(
//     //     &mut self,
//     //     context: &mut ProgramTestContext,
//     //     token_standard: TokenStandard,
//     //     update_authority: &Keypair,
//     //     token_owner: &Pubkey,
//     //     amount: u64,
//     //     payer: &Keypair,
//     //     spl_token_program: Pubkey,
//     // ) -> Result<(), BanksClientError> {
//     //     self.create(
//     //         context,
//     //         String::from("Digial Asset"),
//     //         String::from("https://digital.asset"),
//     //         token_standard,
//     //         *update_authority,
//     //         spl_token_program,
//     //     )
//     //     .await?;

//     //     // self.mint(
//     //     //     context,
//     //     //     token_owner,
//     //     //     amount,
//     //     //     update_authority,
//     //     //     payer,
//     //     //     spl_token_program,
//     //     // )
//     //     // .await?;

//     //     Ok(())
//     // }

//     pub async fn create_default(
//         &mut self,
//         context: &mut ProgramTestContext,
//         token_standard: TokenStandard,
//         spl_token_program: Pubkey,
//     ) -> Result<(), BanksClientError> {
//         let mint_pubkey = self.mint.pubkey();
//         let payer_pubkey = context.payer.pubkey();

//         self.metadata = Metadata::find_pda(&mint_pubkey).0;
//         self.master_edition = MasterEdition::find_pda(&mint_pubkey).0;

//         let create_ix = CreateV1Builder::new()
//             .metadata(self.metadata)
//             .master_edition(Some(self.master_edition))
//             .mint(mint_pubkey, true)
//             .authority(payer_pubkey)
//             .payer(payer_pubkey)
//             .update_authority(payer_pubkey, true)
//             .is_mutable(true)
//             .primary_sale_happened(false)
//             .name(String::from("DigitalAsset"))
//             .uri(String::from("http://digital.asset"))
//             .seller_fee_basis_points(500)
//             .token_standard(token_standard)
//             .print_supply(PrintSupply::Zero)
//             .spl_token_program(Some(spl_token_program))
//             .instruction();

//         let tx = Transaction::new_signed_with_payer(
//             &[create_ix],
//             Some(&context.payer.pubkey()),
//             &[&context.payer, &self.mint],
//             context.last_blockhash,
//         );

//         context.banks_client.process_transaction(tx).await
//     }

//     pub async fn create_default_with_mint_extensions(
//         &mut self,
//         context: &mut ProgramTestContext,
//         token_standard: TokenStandard,
//         extensions: &[ExtensionType],
//     ) -> Result<(), BanksClientError> {
//         let mint_pubkey = self.mint.pubkey();
//         let payer_pubkey = context.payer.pubkey();

//         let token_manager = TokenManager::default();
//         token_manager
//             .create_mint_with_extensions(
//                 context,
//                 &self.mint,
//                 &payer_pubkey,
//                 Some(&payer_pubkey),
//                 0,
//                 extensions,
//             )
//             .await
//             .unwrap();

//         self.metadata = Metadata::find_pda(&mint_pubkey).0;
//         self.master_edition = MasterEdition::find_pda(&mint_pubkey).0;

//         let create_ix = CreateV1Builder::new()
//             .metadata(self.metadata)
//             .master_edition(Some(self.master_edition))
//             .mint(mint_pubkey, true)
//             .authority(payer_pubkey)
//             .payer(payer_pubkey)
//             .update_authority(payer_pubkey, true)
//             .is_mutable(true)
//             .primary_sale_happened(false)
//             .name(String::from("DigitalAsset"))
//             .uri(String::from("http://digital.asset"))
//             .seller_fee_basis_points(500)
//             .token_standard(token_standard)
//             .print_supply(PrintSupply::Zero)
//             .spl_token_program(Some(spl_token_2022::ID))
//             .instruction();

//         let tx = Transaction::new_signed_with_payer(
//             &[create_ix],
//             Some(&context.payer.pubkey()),
//             &[&context.payer, &self.mint],
//             context.last_blockhash,
//         );

//         context.banks_client.process_transaction(tx).await
//     }
// }
