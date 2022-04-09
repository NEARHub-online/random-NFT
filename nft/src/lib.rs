/*!
Non-Fungible Token implementation with JSON serialization.
NOTES:
  - The maximum balance value is limited by U128 (2**128 - 1).
  - JSON calls should pass U128 as a base-10 string. E.g. "100".
  - The contract optimizes the inner trie structure by hashing account IDs. It will prevent some
    abuse of deep tries. Shouldn't be an issue, once NEAR clients implement full hashing of keys.
  - The contract tracks the change in storage before and after the call. If the storage increases,
    the contract requires the caller of the contract to attach enough deposit to the function call
    to cover the storage cost.
    This is done to prevent a denial of service attack on the contract by taking all available storage.
    If the storage decreases, the contract will issue a refund for the cost of the released storage.
    The unused tokens from the attached deposit are also refunded, so it's safe to
    attach more deposit than required.
  - To prevent the deployed contract from being modified or deleted, it should not have any access
    keys on its account.
*/
use near_contract_standards::non_fungible_token::metadata::{
    NFTContractMetadata, NonFungibleTokenMetadataProvider, TokenMetadata, NFT_METADATA_SPEC,
};
use near_contract_standards::non_fungible_token::{Token, TokenId};
use near_contract_standards::non_fungible_token::NonFungibleToken;
use near_sdk::borsh::{self, BorshDeserialize, BorshSerialize};
use near_sdk::collections::{LazyOption};
use near_sdk::json_types::ValidAccountId;
use near_sdk::{
    env, near_bindgen, AccountId, BorshStorageKey, PanicOnDefault, Promise, PromiseOrValue, Gas
};
use near_sdk::serde_json::json;

near_sdk::setup_alloc!();

#[near_bindgen]
#[derive(BorshDeserialize, BorshSerialize, PanicOnDefault)]
pub struct Contract {
    tokens: NonFungibleToken,
    metadata: LazyOption<NFTContractMetadata>,
    pub token_minted: u16,
    pub token_minted_users: u16,
    current_index: u8,
}

const DATA_IMAGE_SVG_NEAR_ICON: &str = "data:image/svg+xml,&lt;svg id=&quot;SVG&quot; xmlns=&quot;http://www.w3.org/2000/svg&quot; width=&quot;500&quot; height=&quot;500&quot; viewBox=&quot;0 0 500 500&quot;&gt;  &lt;defs&gt;    &lt;style&gt;      .cls-1 {        fill: #00b7b2;        stroke: #fff;        stroke-width: 4.43px;      }      .cls-1, .cls-2, .cls-3 {        fill-rule: evenodd;      }      .cls-2 {        fill: #ff6000;      }      .cls-2, .cls-3 {        stroke: #000;        stroke-linejoin: round;        stroke-width: 1px;      }      .cls-3 {        fill: #fff;      }    &lt;/style&gt;  &lt;/defs&gt;  &lt;path class=&quot;cls-1&quot; d=&quot;M8.634,486.008V158.294H282.493l30.012,32.367V158.294H490.076V486.008H197.46L163.7,449.6v36.413H8.634Z&quot;/&gt;  &lt;path id=&quot;Comics_copy_2&quot; data-name=&quot;Comics copy 2&quot; class=&quot;cls-2&quot; d=&quot;M76.724,293.832q-8.535-4.329-21.887-.337-21.015,6.282-35.884,27.189Q4.993,340.24,4.994,359.4q0,17,12.442,26.976,13.275,10.468,32.091,4.843,6.6-1.973,15.552-8.36Q75.4,375.66,75.4,370.531a5.391,5.391,0,0,0-2.579-4.829,7.032,7.032,0,0,0-6.069-.614q-2.884.863-8,4.687t-7.928,4.664q-10.091,3.016-17.373-3.779a20.736,20.736,0,0,1-6.448-15.545q0-11.739,7.511-24.036,8.572-13.695,21.09-17.437,6.6-1.974,6.6,2.479a22.311,22.311,0,0,1-.91,5.062,22.329,22.329,0,0,0-.91,5.062,6.6,6.6,0,0,0,3.262,6.041,9.108,9.108,0,0,0,7.662.678q6.6-1.974,10.621-9.854a28.783,28.783,0,0,0,3.338-13.074Q85.259,298.162,76.724,293.832Zm103.669-4.616q-1.442-13.263-10.773-19.919-10.242-7.326-25.263-2.836-21.621,6.463-37.1,29.981-14.414,21.715-14.414,41.617,0,16.933,12.214,24.885,12.366,8.313,31.181,2.687,22.378-6.691,34.442-28.985a81.56,81.56,0,0,0,10.09-39.852,65.53,65.53,0,0,0-.379-7.578h0Zm-22.684,44.967q-6.6,11.419-17.6,14.707a26.177,26.177,0,0,1-14.377.52,14.757,14.757,0,0,1-9.672-7.093,22.019,22.019,0,0,1-2.352-10.7q0-9.918,5.69-21.232a61.411,61.411,0,0,1,15.021-19.3,26.954,26.954,0,0,0,5.993-2.4,48.78,48.78,0,0,1,6.145-2.714,3.427,3.427,0,0,1,1.745-.05q7.511,1.533,11.608,11.035a41.547,41.547,0,0,1,3.413,17.059,40.223,40.223,0,0,1-5.614,20.164h0ZM272.417,228.444q-7.132,2.133-12.29,11.838-2.049,3.918-6.069,15.644-6.45,18.459-7.663,22.4-4.628,16.834-6.676,25.136-9.711-25.092-14.262-37.833a77.745,77.745,0,0,0-4.7-9.928q-6.6-10.236-16.387-7.311a18.336,18.336,0,0,0-7.928,4.934q-3.526,3.618-3.527,7.531,0,4.318.8,12.715t0.8,12.782q0,8.771-.379,14.888-0.3,4.478-1.973,18.737a164.04,164.04,0,0,0-1.441,18.782q0,11.064,8.724,8.455a11.6,11.6,0,0,0,5.8-3.894,9.337,9.337,0,0,0,2.314-5.954q0-.876-0.114-2.8t-0.114-2.867q0-11.536,5.311-36.8,21.317,40.38,29.359,37.975a9.667,9.667,0,0,0,4.59-3.126,7.231,7.231,0,0,0,2.01-4.649,3.7,3.7,0,0,0-.227-1.147q4.248-9.5,5.917-14.25,7.131-18.12,10.925-27.215,0.606,21,.607,26.2,0,8.433.758,10.972,1.821,5.865,8.573,3.846a12.067,12.067,0,0,0,5.842-3.8,8.914,8.914,0,0,0,2.351-5.864q0-2.293-.189-6.758t-0.19-6.757a130.26,130.26,0,0,1,.986-14.192q1.593-12.687,1.669-14.464,0.91-13.494,1.29-20.288,0.911-9.918.91-11.538,0-15.989-15.4-11.385h0Zm49.16,11.074q-0.226-11.2-1.9-16.164-3.414-8.829-12.594-6.085-9.939,2.971-9.938,13.9,0,2.5,2.807,15.284,2.5,10.99,2.5,31.7,0,5.2-.91,15.822t-0.91,15.823a4.854,4.854,0,0,0,2.2,4.47,6.59,6.59,0,0,0,5.538.368q7.283-2.178,9.331-10.751,0.529-2.385.911-13.293,0.3-6.093,1.669-21.21,1.44-13.72,1.441-21.277,0-2.091-.152-8.59h0Zm80.076-42.83q-8.535-4.331-21.887-.338-21.017,6.283-35.884,27.19-13.961,19.554-13.96,38.714,0,17,12.442,26.977,13.275,10.468,32.091,4.843,6.6-1.974,15.552-8.361,10.317-7.2,10.318-12.327a5.389,5.389,0,0,0-2.58-4.828,7.025,7.025,0,0,0-6.069-.614q-2.883.863-8,4.686t-7.928,4.664q-10.091,3.018-17.222-3.824a20.361,20.361,0,0,1-6.6-15.5q0-11.739,7.511-24.037,8.572-13.693,21.09-17.437,6.6-1.973,6.6,2.48a22.308,22.308,0,0,1-.91,5.062,22.308,22.308,0,0,0-.91,5.062,6.6,6.6,0,0,0,3.262,6.041,9.11,9.11,0,0,0,7.662.677q6.6-1.972,10.621-9.854a28.779,28.779,0,0,0,3.338-13.074Q410.187,201.019,401.653,196.688ZM487.715,169.2q-6.753-3.107-16.311-.25-16.463,4.922-31.18,18.294-16.312,14.862-16.311,28.894,0,10.188,11.531,16.859,6.675,3.942,21.242,8.154,11.455,3.255,11.456,4.873,0,4.251-11.987,7.834a31.085,31.085,0,0,1-15.324.331,7.493,7.493,0,0,0-3.338-.148q-3.264.975-5.842,6.873a32.147,32.147,0,0,0-2.428,8.349q12.517,5.5,27.843.919A54.083,54.083,0,0,0,478,258.123q10.771-9.628,10.773-19.345,0-8.971-10.773-14.522-2.807-1.453-20.18-7.189-10.773-3.593-10.773-7.844,0-4.385,10.09-11.652,8.952-6.453,14.642-8.155a1.175,1.175,0,0,0,.38.021q2.5,9.507,11.91,6.694a14.2,14.2,0,0,0,8.346-6.341A16.2,16.2,0,0,0,495,180.721q0-8.231-7.283-11.518h0Z&quot;/&gt;  &lt;path class=&quot;cls-1&quot; d=&quot;M8.634,148.854V13.992H490.076V148.854H8.634Z&quot;/&gt;  &lt;path id=&quot;NEARHUB&quot; class=&quot;cls-3&quot; d=&quot;M45,128.625v-26.27c0-16.577-.26-30.344-0.912-42h0.391C48.129,70.606,53.34,82.687,57.9,92.24l17.064,36.385H94.241V33.941H77.047V59.508c0,15.312.521,29.08,1.563,40.88H78.35a293.219,293.219,0,0,0-12.9-31.046l-16.8-35.4H27.809v94.685H45ZM145.564,71.308H116.907V50.939h30.22v-17H97.889v94.685h51.062v-17H116.907V88.166h28.657V71.308Zm44.158,33.856,5.862,23.461h19.93L191.285,33.941H166.927l-24.359,94.685h19.278l5.471-23.461h22.405Zm-19.8-15.453,4.56-20.089c1.3-5.479,2.6-13.486,3.777-19.246h0.261c1.3,5.76,2.735,13.627,4.038,19.246l4.689,20.089H169.922Zm43.768,38.914h18.888V91.257H237.4c7.425,0.141,10.942,3.372,13.026,15.172,2.215,11.238,4.3,19.526,5.6,22.2h19.539c-1.694-3.653-4.3-15.734-6.644-26.411-2.084-9.131-5.34-15.312-11.463-17.981V83.811a25.028,25.028,0,0,0,14.98-23.32c0-8.569-2.475-15.172-7.815-19.808-6.123-5.479-14.98-7.445-26.834-7.445a138.445,138.445,0,0,0-24.1,1.967v93.42Zm18.888-79.232A36.249,36.249,0,0,1,240,48.832c8.727,0,13.416,4.917,13.416,13.346,0,8.288-5.34,14.048-14.459,14.048h-6.382V49.393Zm40.511-15.453v94.685h19.018V89.009h28.266v39.616h19.018V33.941H320.373V70.887H292.107V33.941H273.089Zm69.689,0V88.447c0,28.237,11.2,41.582,32.174,41.582,21.233,0,33.347-13.626,33.347-41.441V33.941H389.281V90.554c0,16.015-5.08,22.758-13.808,22.758-8.467,0-13.677-7.164-13.677-22.758V33.941H342.778ZM411.686,128.2a124.742,124.742,0,0,0,19.8,1.405c16.543,0,26.7-3.231,32.826-8.991a26.167,26.167,0,0,0,8.206-19.667c0-11.52-6.773-20.089-16.8-23.039V77.49c9.51-3.933,13.808-11.941,13.808-20.229,0-8.007-3.517-14.47-9.248-18.263-6.253-4.5-13.938-5.76-24.88-5.76-9.118,0-18.5.843-23.707,1.967v93Zm18.887-79.231a26.628,26.628,0,0,1,7.165-.7c8.467,0,12.895,4.355,12.895,11.379,0,7.305-5.21,12.222-14.459,12.222h-5.6v-22.9Zm0,37.789h5.862c8.858,0,16.282,4.074,16.282,13.627,0,9.834-7.294,13.767-15.761,13.767a40.259,40.259,0,0,1-6.383-.281V86.761Z&quot;/&gt;&lt;/svg&gt;";
const MAX_NFT_MINT: u16 = 500;
const MAX_NFT_MINT_USERS: u16 = 300;
const NFT_IMAGES: [&str; 5] = [
        "https://cloudflare-ipfs.com/ipfs/bafybeiejustedpnpl2sl37dvmifszj6xazi6rc7hdulc744nqtkyii7tdi/WL%205%20HRMS%20copy.jpg", 
        "https://cloudflare-ipfs.com/ipfs/bafybeifz7txlqaghmd65xuf3pm6h2sqp2j7szerellxmyxpho74ao7yzcu/WL%204%20HRMS%20copy.jpg", 
        "https://cloudflare-ipfs.com/ipfs/bafybeigllxpu5lwak6hilfojc4dssi43pxajhf3lnichxvut6lwf3ekjsm/WL%203%20HRMS%20copy.jpg", 
        "https://cloudflare-ipfs.com/ipfs/bafybeigrw46fpw3wldc4jdwwpabuift4nk4egkdqdui5dqidfxdon3vgnq/WL%202%20HRMS%20copy.jpg", 
        "https://cloudflare-ipfs.com/ipfs/bafybeie6tdmf5whxd4sy4b7wtnzjafja4pgvlsah2jxknd6wxgtjzqngvy/WL%201%20HRMS%20copy.jpg"];
const NFT_IMAGE_HASHES: [&str; 5] = [
        "3d26f2df03dc554ce08215b208da8047230e350b58784ff94bcc9a24622625f5", 
        "210d372c6d7f08f89478b84c4aabfd9f4991a29198240b06f4102d81dd5bf38d", 
        "cdf42b036d29445986c65d2c271305fa9536738c71249cb53b1682896ee599d6", 
        "87393da5cbdb077e68d4e15a14c423c70e6921cbf5c859792f0ad6a5c7d6b585", 
        "5c9fddd986a2453a482cbd7e541107a023145b7538b6fe0b8c7cbe4fb79dbdfd"
];
const MINT_PRICE: u128 = 5_000_000_000_000_000_000_000_000;
const GAS_RESERVED_FOR_CURRENT_CALL: Gas = 20_000_000_000_000;


#[derive(BorshSerialize, BorshStorageKey)]
enum StorageKey {
    NonFungibleToken,
    Metadata,
    TokenMetadata,
    Enumeration,
    Approval,
}

#[near_bindgen]
impl Contract {
    /// Initializes the contract owned by `owner_id` with
    /// default metadata (for example purposes only).
    #[init]
    pub fn new_default_meta(owner_id: ValidAccountId) -> Self {
        Self::new(
            owner_id,
            NFTContractMetadata {
                spec: NFT_METADATA_SPEC.to_string(),
                name: "Near Hub NFT Comics".to_string(),
                symbol: "NHCOMICS".to_string(),
                icon: Some(DATA_IMAGE_SVG_NEAR_ICON.to_string()),
                base_uri: None,
                reference: None,
                reference_hash: None,
            },
        )
    }

    #[init]
    pub fn new(owner_id: ValidAccountId, metadata: NFTContractMetadata) -> Self {
        assert!(!env::state_exists(), "Already initialized");
        metadata.assert_valid();
        Self {
            tokens: NonFungibleToken::new(
                StorageKey::NonFungibleToken,
                owner_id,
                Some(StorageKey::TokenMetadata),
                Some(StorageKey::Enumeration),
                Some(StorageKey::Approval),
            ),
            metadata: LazyOption::new(StorageKey::Metadata, Some(&metadata)),
            token_minted: 0,
            token_minted_users: 0,
            current_index: 0,
        }
    }

    /// Mint a new token with ID=`token_id` belonging to `receiver_id`.
    ///
    /// Since this example implements metadata, it also requires per-token metadata to be provided
    /// in this call. `self.tokens.mint` will also require it to be Some, since
    /// `StorageKey::TokenMetadata` was provided at initialization.
    ///
    /// `self.tokens.mint` will enforce `predecessor_account_id` to equal the `owner_id` given in
    /// initialization call to `new`.
    #[payable]
    pub fn nft_mint(
        &mut self,
    ) -> Promise {
        assert!(
            env::attached_deposit() >= MINT_PRICE,
            "Attached deposit must be greater than MINT_PRICE"
        );
        assert!(
            self.token_minted < MAX_NFT_MINT,
            "Max token quantity is MAX_NFT_MINT"
        );
        assert!(
            self.token_minted_users < MAX_NFT_MINT_USERS,
            "Max token on sale is MAX_NFT_MINT_USERS"
        );

        let remaining_gas: Gas = env::prepaid_gas() - env::used_gas() - GAS_RESERVED_FOR_CURRENT_CALL;
        Promise::new(env::current_account_id()).function_call(
            b"owner_nft_mint".to_vec(),
            json!({ "receiver_id": env::signer_account_id().to_string() }) // method arguments
                .to_string()
                .into_bytes(),
            75_000_000_000_000_000_000_000,    // amount of yoctoNEAR to attach
            remaining_gas)       // gas to attach)
    }

    #[payable]
    pub fn owner_nft_mint(
        &mut self,
        receiver_id: ValidAccountId,
    ) -> Token {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Only the contract owner can call this method"
        );
        assert!(
            self.token_minted < MAX_NFT_MINT,
            "Max token quantity is MAX_NFT_MINT"
        );
        if self.current_index > 4 {
            self.current_index = 0
        }
        let url = NFT_IMAGES[self.current_index as usize];
        let l: String;
        match self.current_index {
            0 => l = String::from("a"),
            1 => l = String::from("b"),
            2 => l = String::from("c"),
            3 => l = String::from("d"),
            4 => l = String::from("e"),
            _ => l = String::from("e"),
        }
        let title: String =format!("HRMS #1{} Whitelist NFTs", l);
        let _metadata = TokenMetadata {
            title: Some(title.into()),
            description: Some("NFTs created to participate in the whitelist portion of the NEARHUB Comic issue #1 PFP NFT drop.".into()),
            media: Some(url.to_string()),
            media_hash: None,
            copies: Some(100u64),
            issued_at: Some(env::block_timestamp().to_string()),
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        };
        if self.current_index == 4 {
            self.current_index = 0
        }
        else{
            self.current_index += 1;
        }
        self.current_index += 1;
        self.token_minted += 1;
        if env::current_account_id() != env::signer_account_id() {
            self.token_minted_users += 1;
        }
        self.tokens.mint(self.token_minted.to_string(), receiver_id, Some(_metadata))
    }
}

near_contract_standards::impl_non_fungible_token_core!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_approval!(Contract, tokens);
near_contract_standards::impl_non_fungible_token_enumeration!(Contract, tokens);

#[near_bindgen]
impl NonFungibleTokenMetadataProvider for Contract {
    fn nft_metadata(&self) -> NFTContractMetadata {
        self.metadata.get().unwrap()
    }
}

#[cfg(all(test, not(target_arch = "wasm32")))]
mod tests {
    use near_sdk::test_utils::{accounts, VMContextBuilder};
    use near_sdk::testing_env;

    use super::*;

    const MINT_STORAGE_COST: u128 = 5870000000000000000000;

    fn get_context(predecessor_account_id: ValidAccountId) -> VMContextBuilder {
        let mut builder = VMContextBuilder::new();
        builder
            .current_account_id(accounts(0))
            .signer_account_id(predecessor_account_id.clone())
            .predecessor_account_id(predecessor_account_id);
        builder
    }

    fn sample_token_metadata() -> TokenMetadata {
        TokenMetadata {
            title: Some("Olympus Mons".into()),
            description: Some("The tallest mountain in the charted solar system".into()),
            media: None,
            media_hash: None,
            copies: Some(1u64),
            issued_at: None,
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        }
    }

    #[test]
    fn test_new() {
        let mut context = get_context(accounts(1));
        testing_env!(context.build());
        let contract = Contract::new_default_meta(accounts(1).into());
        testing_env!(context.is_view(true).build());
        assert_eq!(contract.nft_token("1".to_string()), None);
    }

    #[test]
    #[should_panic(expected = "The contract is not initialized")]
    fn test_default() {
        let context = get_context(accounts(1));
        testing_env!(context.build());
        let _contract = Contract::default();
    }

    #[test]
    fn test_mint() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());

        let token_id = "0".to_string();
        let token = contract.nft_mint();
        assert_eq!(token.token_id, token_id);
        assert_eq!(token.owner_id, accounts(0).to_string());
        assert_eq!(token.metadata.unwrap(), sample_token_metadata());
        assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
    }

    #[test]
    fn test_transfer() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint();

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_transfer(accounts(1), token_id.clone(), None, None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        if let Some(token) = contract.nft_token(token_id.clone()) {
            assert_eq!(token.token_id, token_id);
            assert_eq!(token.owner_id, accounts(1).to_string());
            assert_eq!(token.metadata.unwrap(), sample_token_metadata());
            assert_eq!(token.approved_account_ids.unwrap(), HashMap::new());
        } else {
            panic!("token not correctly created, or not found by nft_token");
        }
    }

    #[test]
    fn test_approve() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint();

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }

    #[test]
    fn test_revoke() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint();

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke(token_id.clone(), accounts(1));
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), None));
    }

    #[test]
    fn test_revoke_all() {
        let mut context = get_context(accounts(0));
        testing_env!(context.build());
        let mut contract = Contract::new_default_meta(accounts(0).into());

        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(MINT_STORAGE_COST)
            .predecessor_account_id(accounts(0))
            .build());
        let token_id = "0".to_string();
        contract.nft_mint();

        // alice approves bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(150000000000000000000)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_approve(token_id.clone(), accounts(1), None);

        // alice revokes bob
        testing_env!(context
            .storage_usage(env::storage_usage())
            .attached_deposit(1)
            .predecessor_account_id(accounts(0))
            .build());
        contract.nft_revoke_all(token_id.clone());
        testing_env!(context
            .storage_usage(env::storage_usage())
            .account_balance(env::account_balance())
            .is_view(true)
            .attached_deposit(0)
            .build());
        assert!(!contract.nft_is_approved(token_id.clone(), accounts(1), Some(1)));
    }
}
