use {
    crate::error::MintError,
    solana_program::{
        borsh::{try_from_slice_unchecked},
        account_info::{next_account_info, AccountInfo},
        entrypoint::ProgramResult,
        pubkey::Pubkey,
        program,
        program_pack::Pack,
    },
    metaplex_token_metadata::{
        instruction::{create_metadata_accounts, update_metadata_accounts}
    },
    spl_token::{
        state::{
            Account,
            Mint
        },
    },
};

const PREFIX: &str             = "amoebit_minter";
const OUR_PUB_KEY: &str        = "VLawmZTgLAbdeqrU579ohsdey9H1h3Mi1UeUJpg2mQB";

pub fn process_instruction<'a>(
    program_id: &'a Pubkey,
    accounts: &'a [AccountInfo<'a>],
    _input: &[u8],
) -> ProgramResult {
    let accounts_iter           = &mut accounts.iter();

    let payer_account           = next_account_info(accounts_iter)?; // 0
    let rugged_account          = next_account_info(accounts_iter)?; // 1
    let sys_account             = next_account_info(accounts_iter)?; // 2
    let token_account           = next_account_info(accounts_iter)?; // 3
    let mint_account            = next_account_info(accounts_iter)?; // 4
    let meta_account            = next_account_info(accounts_iter)?; // 5
    let meta_program_account    = next_account_info(accounts_iter)?; // 6
    let rent_account            = next_account_info(accounts_iter)?; // 7
    let auth_account            = next_account_info(accounts_iter)?; // 8
    let token_program_account   = next_account_info(accounts_iter)?; // 9
    let rugged_metadata_account = next_account_info(accounts_iter)?; // 10
    let rugged_token_account    = next_account_info(accounts_iter)?; // 11
    let new_update_auth_account = next_account_info(accounts_iter)?; // 12

    if new_update_auth_account.key.to_string() != OUR_PUB_KEY { return Err(MintError::UpdateAuth.into()); }

    let token_data: Account = Pack::unpack(&token_account.data.borrow())?;
    let mint_data: Mint     = Pack::unpack(&mint_account.data.borrow())?;

    // Make sure client sent a proper NFT
    if token_data.amount           != 1                 { return Err(MintError::EmptyToken.into()); }
    if mint_data.decimals          != 0                 { return Err(MintError::InvalidMint.into()); }
    if mint_data.supply            != 1                 { return Err(MintError::InvalidMint.into()); }
    if !mint_data.freeze_authority.is_none()            { return Err(MintError::InvalidMint.into()); }
    if token_data.mint             != *mint_account.key { return Err(MintError::InvalidMint.into()); }

    let auth_seeds = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        PREFIX.as_bytes(),
    ];

    let (auth_key, bump_seed) = 
        Pubkey::find_program_address(auth_seeds, program_id);

    let authority_seeds: &[&[_]] = &[
        PREFIX.as_bytes(),
        program_id.as_ref(),
        PREFIX.as_bytes(),
        &[bump_seed]
    ];

    // safety check (may be not needed because tx will fail(?))
    if auth_key != *auth_account.key {
        return Err(MintError::AuthKeyFailure.into());
    }

    let creators = vec![
        metaplex_token_metadata::state::Creator {
            address: *auth_account.key,
            verified: true,
            share: 0
        },
        metaplex_token_metadata::state::Creator {
            address: *new_update_auth_account.key,
            verified: false,
            share: 100
        },
    ];

    let rugged_data: metaplex_token_metadata::state::Metadata = try_from_slice_unchecked(&rugged_metadata_account.data.borrow())?;

    let cmda_instruction = create_metadata_accounts(
        *meta_program_account.key,
        *meta_account.key,
        *mint_account.key,
        *payer_account.key,
        *payer_account.key,
        *auth_account.key,
        rugged_data.data.name.to_string(),
        rugged_data.data.symbol.to_string(),
        rugged_data.data.uri.to_string(),
        Some(creators),
        500,
        true,
        true
    );

    let metadata_infos = vec![
        meta_account.clone(),
        mint_account.clone(),
        payer_account.clone(),
        meta_program_account.clone(),
        rent_account.clone(),
        auth_account.clone()
    ];

    // create meta data accounts
    program::invoke_signed(
        &cmda_instruction,
        metadata_infos.as_slice(),
        &[&authority_seeds]
    )?;

    let update_infos = vec![
        meta_program_account.clone(),
        meta_account.clone(),
        auth_account.clone(),
    ];

    // denote that the primary sale has happened. set New update authority
    program::invoke_signed(
        &update_metadata_accounts(
            *meta_program_account.key,
            *meta_account.key,
            //*new_update_auth_account.key,
            *auth_account.key,
            None,
            None,
            Some(true),
        ),
        update_infos.as_slice(),
        &[&authority_seeds],
    )?;

    // disable mint
    program::invoke(
        &spl_token::instruction::set_authority(
            &token_program_account.key,
            &mint_account.key,
            None,
            spl_token::instruction::AuthorityType::MintTokens,
            &payer_account.key,
            &[&payer_account.key]
        )?,
        &[
            payer_account.clone(),
            mint_account.clone(),
            token_program_account.clone()
        ]
    )?;

    // burn in hell
    program::invoke(
        &spl_token::instruction::burn(
            &token_program_account.key,
            &rugged_token_account.key,
            &rugged_account.key,
            &payer_account.key,
            &[&payer_account.key],
            1
        )?,
        &[
            token_program_account.clone(),
            rugged_token_account.clone(),
            rugged_account.clone(),
            payer_account.clone(),
        ]
    )?;

    Ok(())
}
