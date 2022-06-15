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
    ) -> Promise {
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
            "nft_mint_owner".to_string(),
            json!({ "receiver_id": env::signer_account_id().to_string() }) // method arguments
                .to_string()
                .into_bytes(),
            75_000_000_000_000_000_000_000,    // amount of yoctoNEAR to attach
            remaining_gas)       // gas to attach)
    }


    #[payable]
    pub fn nft_mint_owner(
        &mut self,
        receiver_id: AccountId,
    ) {
        assert_eq!(
            env::predecessor_account_id(),
            env::current_account_id(),
            "Only the contract owner can call this method"
        );
        assert!(
            self.token_minted < MAX_NFT_MINT,
            "Max token quantity is MAX_NFT_MINT"
        );
        
        let _metadata = TokenMetadata {
            title: Some("GemZ: NFT NYC 2022".into()),
            description: Some("GemZ is a 2D and 3D generative project consisting of a limited edition 1,111 NFTs by Saint Kyriaki and TiEn. Ethics, morals, and values are at the heart of GemZ. In our lives, GemZ are the people we admire and respect. GemZ are our spiritual guides and mentors. GemZ are people who add a dash of whimsy and color to everyday life. GemZ is the stuff of our dreams and aspirations. GemZ is what we'd like to have in our back pocket to keep us safe and sacred, playful, and fun. A lot of brands, projects, and things that we see in the world are the work of GemZ. They don't get the notoriety and the percentages. Our project honors them and gives them acclamation. They've been swept under the rug, and no one seems to notice. Like a diamond tucked away in a pile of soil , GemZ are the undiscovered treasures. The core of the brand is provided by these artists that on the other hand, are often shunned and not given the attention they deserve. In many cases, artists feel that they are not given a fair share of the profits, are overlooked, and are not compensated in a way that is equitable. GemZ is all about honouring one's talents and breaking the stigma of getting the art from the talent and sweeping them under the rug. GemZ is a statement. It is our mission to make the GemZ more widely known, heard, and seen.".into()),
            media: Some(NFT_IMAGES.to_string()),
            media_hash: None,
            copies: Some(420u64),
            issued_at: Some(env::block_timestamp()),
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
            reference: None,
            reference_hash: None,
        };
        self.token_minted += 1;
        if env::current_account_id() != env::signer_account_id() {
            self.token_minted_users += 1;
        }

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
            self.tokens_by_id.insert(&self.token_minted.to_string(), &token).is_none(),
            "Token already exists"
        );

        //insert the token ID and metadata
        self.token_metadata_by_id.insert(&self.token_minted.to_string(), &_metadata);

        //call the internal method for adding the token to the owner
        self.internal_add_token_to_owner(&token.owner_id, &self.token_minted.to_string());

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