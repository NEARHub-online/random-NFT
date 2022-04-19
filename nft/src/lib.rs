use std::collections::HashMap;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption, LookupMap, UnorderedMap, UnorderedSet};
use near_sdk::json_types::{Base64VecU8, U128};
use near_sdk::serde::{Deserialize, Serialize};
use near_sdk::serde_json::json;
use near_sdk::{
    env, near_bindgen, AccountId, Balance, CryptoHash, PanicOnDefault, Promise, PromiseOrValue, Gas, ext_contract
};

use crate::internal::*;
pub use crate::metadata::*;
pub use crate::mint::*;
pub use crate::nft_core::*;
pub use crate::approval::*;
pub use crate::royalty::*;
pub use crate::events::*;

mod internal;
mod approval; 
mod enumeration; 
mod metadata; 
mod mint; 
mod nft_core; 
mod royalty; 
mod events;

/// This spec can be treated like a version of the standard.
pub const NFT_METADATA_SPEC: &str = "1.0.0";
/// This is the name of the NFT standard we're using
pub const NFT_STANDARD_NAME: &str = "nep171";

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,%3Csvg id='SVG' xmlns='http://www.w3.org/2000/svg' width='500' height='500'%3E%%3Cpath class='cls-1' fill='%2300b7b2' stroke='%23fff' stroke-width='4.43px' fill-rule='evenodd' d='M8.634,486.008V158.294H282.493l30.012,32.367V158.294H490.076V486.008H197.46L163.7,449.6v36.413H8.634Z'/%3E%3Cpath id='Comics_copy_2' data-name='Comics copy 2' class='cls-2' stroke='%23000' stroke-linejoin='round' stroke-width='1px' fill='%23ff6000' d='M76.724,293.832q-8.535-4.329-21.887-.337-21.015,6.282-35.884,27.189Q4.993,340.24,4.994,359.4q0,17,12.442,26.976,13.275,10.468,32.091,4.843,6.6-1.973,15.552-8.36Q75.4,375.66,75.4,370.531a5.391,5.391,0,0,0-2.579-4.829,7.032,7.032,0,0,0-6.069-.614q-2.884.863-8,4.687t-7.928,4.664q-10.091,3.016-17.373-3.779a20.736,20.736,0,0,1-6.448-15.545q0-11.739,7.511-24.036,8.572-13.695,21.09-17.437,6.6-1.974,6.6,2.479a22.311,22.311,0,0,1-.91,5.062,22.329,22.329,0,0,0-.91,5.062,6.6,6.6,0,0,0,3.262,6.041,9.108,9.108,0,0,0,7.662.678q6.6-1.974,10.621-9.854a28.783,28.783,0,0,0,3.338-13.074Q85.259,298.162,76.724,293.832Zm103.669-4.616q-1.442-13.263-10.773-19.919-10.242-7.326-25.263-2.836-21.621,6.463-37.1,29.981-14.414,21.715-14.414,41.617,0,16.933,12.214,24.885,12.366,8.313,31.181,2.687,22.378-6.691,34.442-28.985a81.56,81.56,0,0,0,10.09-39.852,65.53,65.53,0,0,0-.379-7.578h0Zm-22.684,44.967q-6.6,11.419-17.6,14.707a26.177,26.177,0,0,1-14.377.52,14.757,14.757,0,0,1-9.672-7.093,22.019,22.019,0,0,1-2.352-10.7q0-9.918,5.69-21.232a61.411,61.411,0,0,1,15.021-19.3,26.954,26.954,0,0,0,5.993-2.4,48.78,48.78,0,0,1,6.145-2.714,3.427,3.427,0,0,1,1.745-.05q7.511,1.533,11.608,11.035a41.547,41.547,0,0,1,3.413,17.059,40.223,40.223,0,0,1-5.614,20.164h0ZM272.417,228.444q-7.132,2.133-12.29,11.838-2.049,3.918-6.069,15.644-6.45,18.459-7.663,22.4-4.628,16.834-6.676,25.136-9.711-25.092-14.262-37.833a77.745,77.745,0,0,0-4.7-9.928q-6.6-10.236-16.387-7.311a18.336,18.336,0,0,0-7.928,4.934q-3.526,3.618-3.527,7.531,0,4.318.8,12.715t0.8,12.782q0,8.771-.379,14.888-0.3,4.478-1.973,18.737a164.04,164.04,0,0,0-1.441,18.782q0,11.064,8.724,8.455a11.6,11.6,0,0,0,5.8-3.894,9.337,9.337,0,0,0,2.314-5.954q0-.876-0.114-2.8t-0.114-2.867q0-11.536,5.311-36.8,21.317,40.38,29.359,37.975a9.667,9.667,0,0,0,4.59-3.126,7.231,7.231,0,0,0,2.01-4.649,3.7,3.7,0,0,0-.227-1.147q4.248-9.5,5.917-14.25,7.131-18.12,10.925-27.215,0.606,21,.607,26.2,0,8.433.758,10.972,1.821,5.865,8.573,3.846a12.067,12.067,0,0,0,5.842-3.8,8.914,8.914,0,0,0,2.351-5.864q0-2.293-.189-6.758t-0.19-6.757a130.26,130.26,0,0,1,.986-14.192q1.593-12.687,1.669-14.464,0.91-13.494,1.29-20.288,0.911-9.918.91-11.538,0-15.989-15.4-11.385h0Zm49.16,11.074q-0.226-11.2-1.9-16.164-3.414-8.829-12.594-6.085-9.939,2.971-9.938,13.9,0,2.5,2.807,15.284,2.5,10.99,2.5,31.7,0,5.2-.91,15.822t-0.91,15.823a4.854,4.854,0,0,0,2.2,4.47,6.59,6.59,0,0,0,5.538.368q7.283-2.178,9.331-10.751,0.529-2.385.911-13.293,0.3-6.093,1.669-21.21,1.44-13.72,1.441-21.277,0-2.091-.152-8.59h0Zm80.076-42.83q-8.535-4.331-21.887-.338-21.017,6.283-35.884,27.19-13.961,19.554-13.96,38.714,0,17,12.442,26.977,13.275,10.468,32.091,4.843,6.6-1.974,15.552-8.361,10.317-7.2,10.318-12.327a5.389,5.389,0,0,0-2.58-4.828,7.025,7.025,0,0,0-6.069-.614q-2.883.863-8,4.686t-7.928,4.664q-10.091,3.018-17.222-3.824a20.361,20.361,0,0,1-6.6-15.5q0-11.739,7.511-24.037,8.572-13.693,21.09-17.437,6.6-1.973,6.6,2.48a22.308,22.308,0,0,1-.91,5.062,22.308,22.308,0,0,0-.91,5.062,6.6,6.6,0,0,0,3.262,6.041,9.11,9.11,0,0,0,7.662.677q6.6-1.972,10.621-9.854a28.779,28.779,0,0,0,3.338-13.074Q410.187,201.019,401.653,196.688ZM487.715,169.2q-6.753-3.107-16.311-.25-16.463,4.922-31.18,18.294-16.312,14.862-16.311,28.894,0,10.188,11.531,16.859,6.675,3.942,21.242,8.154,11.455,3.255,11.456,4.873,0,4.251-11.987,7.834a31.085,31.085,0,0,1-15.324.331,7.493,7.493,0,0,0-3.338-.148q-3.264.975-5.842,6.873a32.147,32.147,0,0,0-2.428,8.349q12.517,5.5,27.843.919A54.083,54.083,0,0,0,478,258.123q10.771-9.628,10.773-19.345,0-8.971-10.773-14.522-2.807-1.453-20.18-7.189-10.773-3.593-10.773-7.844,0-4.385,10.09-11.652,8.952-6.453,14.642-8.155a1.175,1.175,0,0,0,.38.021q2.5,9.507,11.91,6.694a14.2,14.2,0,0,0,8.346-6.341A16.2,16.2,0,0,0,495,180.721q0-8.231-7.283-11.518h0Z'/%3E%3Cpath class='cls-1' fill='%2300b7b2' stroke='%23fff' stroke-width='4.43px' fill-rule='evenodd'  d='M8.634,148.854V13.992H490.076V148.854H8.634Z'/%3E%3Cpath id='NEARHUB' class='cls-3' stroke='%23000' stroke-linejoin='round' stroke-width='1px' fill='%23fff' d='M45,128.625v-26.27c0-16.577-.26-30.344-0.912-42h0.391C48.129,70.606,53.34,82.687,57.9,92.24l17.064,36.385H94.241V33.941H77.047V59.508c0,15.312.521,29.08,1.563,40.88H78.35a293.219,293.219,0,0,0-12.9-31.046l-16.8-35.4H27.809v94.685H45ZM145.564,71.308H116.907V50.939h30.22v-17H97.889v94.685h51.062v-17H116.907V88.166h28.657V71.308Zm44.158,33.856,5.862,23.461h19.93L191.285,33.941H166.927l-24.359,94.685h19.278l5.471-23.461h22.405Zm-19.8-15.453,4.56-20.089c1.3-5.479,2.6-13.486,3.777-19.246h0.261c1.3,5.76,2.735,13.627,4.038,19.246l4.689,20.089H169.922Zm43.768,38.914h18.888V91.257H237.4c7.425,0.141,10.942,3.372,13.026,15.172,2.215,11.238,4.3,19.526,5.6,22.2h19.539c-1.694-3.653-4.3-15.734-6.644-26.411-2.084-9.131-5.34-15.312-11.463-17.981V83.811a25.028,25.028,0,0,0,14.98-23.32c0-8.569-2.475-15.172-7.815-19.808-6.123-5.479-14.98-7.445-26.834-7.445a138.445,138.445,0,0,0-24.1,1.967v93.42Zm18.888-79.232A36.249,36.249,0,0,1,240,48.832c8.727,0,13.416,4.917,13.416,13.346,0,8.288-5.34,14.048-14.459,14.048h-6.382V49.393Zm40.511-15.453v94.685h19.018V89.009h28.266v39.616h19.018V33.941H320.373V70.887H292.107V33.941H273.089Zm69.689,0V88.447c0,28.237,11.2,41.582,32.174,41.582,21.233,0,33.347-13.626,33.347-41.441V33.941H389.281V90.554c0,16.015-5.08,22.758-13.808,22.758-8.467,0-13.677-7.164-13.677-22.758V33.941H342.778ZM411.686,128.2a124.742,124.742,0,0,0,19.8,1.405c16.543,0,26.7-3.231,32.826-8.991a26.167,26.167,0,0,0,8.206-19.667c0-11.52-6.773-20.089-16.8-23.039V77.49c9.51-3.933,13.808-11.941,13.808-20.229,0-8.007-3.517-14.47-9.248-18.263-6.253-4.5-13.938-5.76-24.88-5.76-9.118,0-18.5.843-23.707,1.967v93Zm18.887-79.231a26.628,26.628,0,0,1,7.165-.7c8.467,0,12.895,4.355,12.895,11.379,0,7.305-5.21,12.222-14.459,12.222h-5.6v-22.9Zm0,37.789h5.862c8.858,0,16.282,4.074,16.282,13.627,0,9.834-7.294,13.767-15.761,13.767a40.259,40.259,0,0,1-6.383-.281V86.761Z'/%3E%3C/svg%3E";
const MAX_NFT_MINT: u16 = 1680;
const MAX_NFT_MINT_USERS: u16 = 300;
const NFT_IMAGES: &str = "https://cloudflare-ipfs.com/ipfs/QmcsNiFBkXMkabtmQKurMimDxSi5JumuAsRgodpBZSMjJd/";

const MINT_PRICE: u128 = 420_000_000_000_000_000_000_000;
const GAS_RESERVED_FOR_CURRENT_CALL: Gas = Gas(20_000_000_000_000);

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    //contract owner
    pub owner_id: AccountId,

    //keeps track of all the token IDs for a given account
    pub tokens_per_owner: LookupMap<AccountId, UnorderedSet<TokenId>>,

    //keeps track of the token struct for a given token ID
    pub tokens_by_id: LookupMap<TokenId, Token>,

    //keeps track of the token metadata for a given token ID
    pub token_metadata_by_id: UnorderedMap<TokenId, TokenMetadata>,

    //keeps track of the metadata for the contract
    pub metadata: LazyOption<NFTContractMetadata>,
    pub token_minted: u16,
    pub token_minted_users: u16,
    current_index: u8,
    pub perpetual_royalties: UnorderedMap<AccountId, u32>,
    pub receiver_id: AccountId,
}

/// Helper structure for keys of the persistent collections.
#[derive(BorshSerialize)]
pub enum StorageKey {
    TokensPerOwner,
    TokenPerOwnerInner { account_id_hash: CryptoHash },
    TokensById,
    TokenMetadataById,
    NFTContractMetadata,
    TokensPerType,
    TokensPerTypeInner { token_type_hash: CryptoHash },
    TokenTypesLocked,
    Royalties,
}

#[near_bindgen]
impl Contract {
    /*
        initialization function (can only be called once).
        this initializes the contract with default metadata so the
        user doesn't have to manually type metadata.
    */
    #[init]
    pub fn new_default_meta(owner_id: AccountId, receiver_id: AccountId) -> Self {
        assert!(
            env::is_valid_account_id(owner_id.as_bytes()),
            "The owner account ID is invalid"
        );
        assert!(
            env::is_valid_account_id(receiver_id.as_bytes()),
            "The receiver account ID is invalid"
        );
        //calls the other function "new: with some default metadata and the owner_id passed in 
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: "nft-1.0.0".to_string(),
                name: "Near Hub - OMMM NFT Comics".to_string(),
                symbol: "OMMM".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
            receiver_id,
        )
    }

    /*
        initialization function (can only be called once).
        this initializes the contract with metadata that was passed in and
        the owner_id. 
    */
    #[init]
    pub fn new(owner_id: AccountId, metadata: NFTContractMetadata, receiver_id: AccountId) -> Self {
        //create a variable of type Self with all the fields initialized. 
        let mut this = Self {
            //Storage keys are simply the prefixes used for the collections. This helps avoid data collision
            tokens_per_owner: LookupMap::new(StorageKey::TokensPerOwner.try_to_vec().unwrap()),
            tokens_by_id: LookupMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            token_metadata_by_id: UnorderedMap::new(
                StorageKey::TokenMetadataById.try_to_vec().unwrap(),
            ),
            //set the owner_id field equal to the passed in owner_id. 
            owner_id,
            metadata: LazyOption::new(
                StorageKey::NFTContractMetadata.try_to_vec().unwrap(),
                Some(&metadata),
            ),
            token_minted: 0,
            token_minted_users: 0,
            current_index: 0,
            perpetual_royalties: UnorderedMap::new(StorageKey::TokensById.try_to_vec().unwrap()),
            receiver_id: receiver_id.clone().into(),
        };

        this.perpetual_royalties.insert(&receiver_id.clone().into(), &2000);

        //return the Contract object
        this
    }
}