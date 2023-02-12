use crate::*;

// One can provide a name, e.g. `ext` to use for generated methods.
#[ext_contract(ext)]
pub trait ExtCrossWhitelist {
    fn on_get_whitelist(&self, #[callback_unwrap] quantity: U128) -> U128;
}

#[near_bindgen]
impl Contract {
    #[payable]
    pub fn nft_mint(
        &mut self,
        image: String,
        asset: String,
    ) -> Promise {
        assert!(
            env::attached_deposit() >= MINT_PRICE,
            "Attached deposit must be greater than MINT_PRICE"
        );
        assert!(
            self.token_minted < MAX_NFT_MINT,
            "Max token quantity is MAX_NFT_MINT"
        );

        let remaining_gas: Gas = env::prepaid_gas() - env::used_gas() - GAS_RESERVED_FOR_CURRENT_CALL;
        Promise::new(env::current_account_id()).function_call(
            "nft_mint_owner".to_string(),
            json!({ "receiver_id": env::signer_account_id().to_string(), "image": image, "asset": asset }) // method arguments
                .to_string()
                .into_bytes(),
            75_000_000_000_000_000_000_000,    // amount of yoctoNEAR to attach
            remaining_gas)       // gas to attach)
    }


    #[payable]
    pub fn nft_mint_owner(
        &mut self,
        receiver_id: AccountId,
        image: String,
        asset: String,
    ) {
        assert!(
            env::is_valid_account_id(receiver_id.as_bytes()),
            "The receiver account ID is invalid"
        );
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Only the contract owner can call this method"
        );
        assert!(
            self.token_minted < MAX_NFT_MINT,
            "Max token quantity is MAX_NFT_MINT"
        );
        let title = format!("NEAR Avatar #{}", self.token_minted);
        let asset_url = format!("{}{}", IPFS_GATEWAY, asset);
        let image_url = format!("{}{}", IPFS_GATEWAY, image);
        let extra = json!({ "asset": asset_url });
        let _metadata = TokenMetadata {
            title: Some(title.into()),
            description: Some("NEAR World Order limited edition NFTs created for NEARCon attendees. This NFT represents the beginning of The NEAR World Order. The NEAR World Order of blockchain is here; mass adoption is NEAR!
            *10 Lucky holders who get the Hollywood Logan NFT, can claim a Tshirt from the NEAR Hub booth at NEAR Con 2022. *".into()),
            media: Some(image_url.to_string()),
            media_hash: None,
            copies: Some(999u64),
            issued_at: Some(env::block_timestamp()),
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra:Some(extra.to_string()),
            reference: None,
            reference_hash: None,
        };
        self.token_minted += 1;

        let mut royalty = HashMap::new();

        //make sure that the length of the perpetual royalties is below 7 since we won't have enough GAS to pay out that many people
        assert!(self.perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty amounts");

        //iterate through the perpetual royalties and insert the account and amount in the royalty map
        for (account, amount) in self.perpetual_royalties.to_vec() {
            royalty.insert(account, amount);
        }

        //specify the token struct that contains the owner ID 
        let token = Token {
            //set the owner ID equal to the receiver ID passed into the function
            owner_id: receiver_id,
            //we set the approved account IDs to the default value (an empty map)
            approved_account_ids: Default::default(),
            //the next approval ID is set to 0
            next_approval_id: 0,
            //the map of perpetual royalties for the token (The owner will get 100% - total perpetual royalties)
            royalty,
        };

        //insert the token ID and token struct and make sure that the token doesn't exist
        assert!(
            self.tokens_by_id.insert(&asset.to_string(), &token).is_none(),
            "Token already exists"
        );

        //insert the token ID and metadata
        self.token_metadata_by_id.insert(&asset.to_string(), &_metadata);

        //call the internal method for adding the token to the owner
        self.internal_add_token_to_owner(&token.owner_id, &asset.to_string());

        // Construct the mint log as per the events standard.
        let nft_mint_log: EventLog = EventLog {
            // Standard name ("nep171").
            standard: NFT_STANDARD_NAME.to_string(),
            // Version of the standard ("nft-1.0.0").
            version: NFT_METADATA_SPEC.to_string(),
            // The data related with the event stored in a vector.
            event: EventLogVariant::NftMint(vec![NftMintLog {
                // Owner of the token.
                owner_id: token.owner_id.to_string(),
                // Vector of token IDs that were minted.
                token_ids: vec![self.token_minted.to_string()],
                // An optional memo to include.
                memo: None,
            }]),
        };

        // Log the serialized json.
        env::log_str(&nft_mint_log.to_string());
    }

}