use crate::*;

#[near_bindgen]
impl Contract {
    #[payable]
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
            b"nft_mint_owner".to_vec(),
            json!({ "receiver_id": env::signer_account_id().to_string() }) // method arguments
                .to_string()
                .into_bytes(),
            75_000_000_000_000_000_000_000,    // amount of yoctoNEAR to attach
            remaining_gas)       // gas to attach)
    }


    #[payable]
    pub fn nft_mint_owner(
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

        let mut royalty = HashMap::new();

        // if perpetual royalties were passed into the function: 
        if let Some(perpetual_royalties) = perpetual_royalties {
            //make sure that the length of the perpetual royalties is below 7 since we won't have enough GAS to pay out that many people
            assert!(perpetual_royalties.len() < 7, "Cannot add more than 6 perpetual royalty amounts");

            //iterate through the perpetual royalties and insert the account and amount in the royalty map
            for (account, amount) in perpetual_royalties {
                royalty.insert(account, amount);
            }
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
        self.token_metadata_by_id.insert(&self.token_minted.to_string(), &metadata);

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