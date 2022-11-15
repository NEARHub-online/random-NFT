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
    ) {
        assert!(
            self.token_minted < MAX_NFT_MINT,
            "Max token quantity is MAX_NFT_MINT"
        );

        let mut mint_price: u128 = 0;

        if self.token_minted < 100 {
            mint_price = MINT_PRICE_1;
            
            assert!(
                self.tokens_per_owner.contains_key(&env::signer_account_id()) == false,
                "One token per account. you already hold a token."
            );
        }

        if self.token_minted >= 100 && self.token_minted < 300 {
            mint_price = MINT_PRICE_2;
        }

        if self.token_minted >= 300 && self.token_minted < 500 {
            mint_price = MINT_PRICE_3;
        }

        if self.token_minted >= 500 {
            mint_price = MINT_PRICE_4;
        } 
        
        assert!(
            env::attached_deposit() >= mint_price,
            "Attached deposit must be greater than mint_price"
        );

        let receive_account_id = env::signer_account_id();
        
        let _metadata = TokenMetadata {
            title: Some("Animaze Animation Film festival 2022 Virtual Ticket".into()),
            description: Some("Animaze is an international animation film festival that started in 2013. It's produced by the Montreal International Animation Film Festival and conference, and it showcases animations of all different kinds from all over the world. 

            This year, it's coming to NEAR Hub for the first time ever! iFeel Virtuel will be streaming the live festival in 360 video, so you can experience it all from the comfort of your own virtual theater. This nft will grant you access to all the amazing animations and talks that'll be happening throughout the event.
            
            Experience this one-of-a-kind opportunity to explore the world of animation live Nov. 18 -20! You'll be able to see all sorts of different animations created just for this festival. It's sure to be a fascinating experience for everyone involved!".into()),
            media: Some(NFT_IMAGES.to_string()),
            media_hash: None,
            copies: Some(2000u64),
            issued_at: Some(env::block_timestamp()),
            expires_at: None,
            starts_at: None,
            updated_at: None,
            extra: None,
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
            owner_id: receive_account_id,
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