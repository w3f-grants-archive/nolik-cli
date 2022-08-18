#[cfg(test)]
mod message {
    use std::fs;
    use std::io::Write;
    use async_std;
    use blake2::Digest;
    use blake2::digest::Update;
    use rand::distributions::Alphanumeric;
    use rand::Rng;
    use sodiumoxide::crypto::box_;
    use sp_core::crypto::AccountId32;
    use sp_keyring::AccountKeyring;
    use nolik_cli::account::{Account, AccountInput, AccountOutput};
    use nolik_cli::blacklist::Blacklist;
    use nolik_cli::cli::config::{Config, ConfigFile};
    use nolik_cli::cli::input::Input;
    use nolik_cli::message::input::BatchInput;
    use nolik_cli::cli::errors::InputError;
    use nolik_cli::message::batch::Batch;
    use nolik_cli::message::ipfs::IpfsInput;
    use nolik_cli::message::message::EncryptedMessage;
    use nolik_cli::message::session::{Session};
    use nolik_cli::message::utils::{base64_to_nonce, base64_to_public_key};
    use nolik_cli::node::errors::NodeError;
    use nolik_cli::node::events::{BalanceTransferEvent, NodeEvent};
    use nolik_cli::node::extrinsics::balance_transfer;
    use nolik_cli::owner::Owner;
    use nolik_cli::wallet::{Wallet, WalletInput};
    use nolik_cli::whitelist::Whitelist;

    #[test]
    fn required_arguments_are_not_provided() {
        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let message_input = Input::new(args).unwrap_err();

        assert_eq!(
            message_input,
            InputError::RequiredKeysMissing
        );
    }

    #[test]
    fn broken_key_value_arguments() {
        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            "bob",
            "--key",
            "message",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let message_input = Input::new(args).unwrap_err();

        assert_eq!(
            message_input,
            InputError::NoCorrespondingValue
        );
    }

    #[test]
    fn sender_does_not_exist() {
        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            "bob",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        let message_input = BatchInput::new(&mut input, &config_file).unwrap_err();

        assert_eq!(
            message_input,
            InputError::SenderDoesNotExist,
        );
    }


    #[test]
    fn sender_name_exist() {
        let arr = [
            "add",
            "account",
            "--alias",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, &account).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            "Gq5xd5c62w4fryJx8poYexoBJAy9JUpjir9vR4qMDF6z",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let message_input = BatchInput::new(&mut input, &config_file).unwrap();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            message_input.sender.alias,
            "alice".to_string(),
        );
    }

    #[test]
    fn sender_address_exist() {
        let arr = [
            "add",
            "account",
            "--alias",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap();
        let account_address = account.public.clone();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, &account).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            format!("{}", bs58::encode(&account_address).into_string()).as_str(),
            "--recipient",
            "Gq5xd5c62w4fryJx8poYexoBJAy9JUpjir9vR4qMDF6z",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let message_input = BatchInput::new(&mut input, &config_file).unwrap();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            message_input.sender.public,
            account_address,
        );
    }


    #[test]
    fn broken_recipient_address() {
        let arr = [
            "add",
            "account",
            "--alias",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let account = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, &account).unwrap();


        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            "@q5xd5c62w4fryJx8poYexoBJAy9JUpjir9vR4qMDF6z",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let message_input = BatchInput::new(&mut input, &config_file).unwrap_err();

        fs::remove_file(config_file.path).unwrap();

        assert_eq!(
            message_input,
            InputError::InvalidAddress,
        )
    }


    async fn generate_message_input() -> (Vec<Account>, BatchInput) {
        let arr = [
            "add",
            "account",
            "--alias",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let alice = Account::new(account_input).unwrap();

        let config_file: ConfigFile = ConfigFile::temp();
        Account::add(&config_file, &alice).unwrap();

        let arr = [
            "add",
            "account",
            "--alias",
            "bob",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let bob = Account::new(account_input).unwrap();

        let arr = [
            "add",
            "account",
            "--alias",
            "carol",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let carol = Account::new(account_input).unwrap();

        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();

        let file_name = format!("temp_{}_text.txt", s);

        let home_dir = dirs::home_dir().unwrap();
        let home_path = home_dir.as_path();
        let nolik_dir = home_path.join(".nolik");
        let text_file = nolik_dir.join(file_name.as_str());

        if let Ok(mut file) = fs::File::create(&text_file) {
            let contents = "Hello World";
            file.write_all(contents.as_ref()).unwrap();
        }


        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--recipient",
            format!("{}", bs58::encode(&carol.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
            "--key",
            "message",
            "--value",
            "test",
            "--file",
            format!("{}", &text_file.clone().into_os_string().into_string().unwrap()).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let bi = BatchInput::new(&mut input, &config_file).unwrap();
        let recipients: Vec<Account> = vec![bob, carol];

        fs::remove_file(config_file.path).unwrap();
        fs::remove_file(text_file).unwrap();

        (recipients, bi)
    }


    #[async_std::test]
    async fn message_decrypted_by_sender() {
        let (_recipient, bi) = generate_message_input().await;

        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();
        let ipfs_data = ipfs_file.get().await.unwrap();

        let public_nonce = base64_to_nonce(&ipfs_data.nonce).unwrap();
        let broker = base64_to_public_key(&ipfs_data.broker).unwrap();

        let decrypted_sessions: Vec<Session> = ipfs_data.sessions
            .iter()
            .filter_map(|es| es.decrypt(&public_nonce, &broker, &bi.sender.secret).ok())
            .collect();

        let first_session = decrypted_sessions.first().unwrap();
        let first_address = first_session.group.0.first().unwrap();

        assert_eq!(
            first_session.nonce.0,
            secret_nonce,
        );

        assert_eq!(
            first_address.0,
            bi.sender.public,
        );

        let recipients = first_session.group.get_recipients();
        let any_recipient = recipients.first().unwrap();

        let mut parties = blake2::Blake2s256::new();
        Update::update(&mut parties, &first_address.0.as_ref());
        Update::update(&mut parties, &any_recipient.as_ref());
        let parties_hash = base64::encode(parties.finalize().to_vec());

        let encrypted_messages = ipfs_data.messages
            .iter()
            .filter(|em| em.parties == parties_hash)
            .collect::<Vec<&EncryptedMessage>>();

        let encrypted_message = encrypted_messages.first().unwrap();
        let decrypted_message = encrypted_message.decrypt(first_session, any_recipient, &bi.sender.secret).unwrap();

        assert_eq!(
            decrypted_message.entries.first().unwrap().key,
            bi.entries.first().unwrap().key,
        );

        assert_eq!(
            decrypted_message.entries.first().unwrap().value,
            bi.entries.first().unwrap().value,
        );
    }


    #[async_std::test]
    async fn message_decrypted_by_recipients() {
        let (recipients, bi) = generate_message_input().await;

        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();
        let ipfs_data = ipfs_file.get().await.unwrap();

        let public_nonce = base64_to_nonce(&ipfs_data.nonce).unwrap();
        let broker = base64_to_public_key(&ipfs_data.broker).unwrap();

        for recipient in &recipients {
            let decrypted_sessions: Vec<Session> = ipfs_data.sessions
                .iter()
                .filter_map(|es| es.decrypt(&public_nonce, &broker, &recipient.secret).ok())
                .collect::<Vec<Session>>();

            let first_session = decrypted_sessions.first().unwrap();
            let first_address = first_session.group.0.first().unwrap();

            assert_eq!(
                first_session.nonce.0,
                secret_nonce,
            );

            assert_eq!(
                first_address.0,
                bi.sender.public,
            );

            assert_eq!(
                first_session.group.0.iter().any(|el| el.0 == recipient.public),
                true,
            );

            let mut parties = blake2::Blake2s256::new();
            Update::update(&mut parties, &first_address.0.as_ref());
            Update::update(&mut parties, &recipient.public.as_ref());
            let parties_hash = base64::encode(parties.finalize().to_vec());

            let encrypted_messages = ipfs_data.messages
                .iter()
                .filter(|em| em.parties == parties_hash)
                .collect::<Vec<&EncryptedMessage>>();

            let encrypted_message = encrypted_messages.first().unwrap();
            let decrypted_message = encrypted_message.decrypt(first_session, &first_address.0, &recipient.secret).unwrap();

            assert_eq!(
                decrypted_message.entries.first().unwrap().key,
                bi.entries.first().unwrap().key,
            );

            assert_eq!(
                decrypted_message.entries.first().unwrap().value,
                bi.entries.first().unwrap().value,
            );
        }
    }


    #[async_std::test]
    async fn confirmed_batch_hash() {
        let (.., bi) = generate_message_input().await;

        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();
        let ipfs_data = ipfs_file.get().await.unwrap();

        let batch_hash = Batch::hash(&bi, &secret_nonce);
        assert_eq!(
            batch_hash,
            ipfs_data.hash,
        )
    }


    async fn init_sending() -> ConfigFile {
        let config_file: ConfigFile = ConfigFile::temp();

        let arr = [
            "add",
            "account",
            "--alias",
            "alice",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let alice = Account::new(account_input).unwrap();
        Account::add(&config_file, &alice).unwrap();


        let arr = [
            "add",
            "account",
            "--alias",
            "bob",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let bob = Account::new(account_input).unwrap();
        Account::add(&config_file, &bob).unwrap();


        let arr = [
            "add",
            "account",
            "--alias",
            "carol",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let account_input = AccountInput::new(input).unwrap();
        let carol = Account::new(account_input).unwrap();
        Account::add(&config_file, &carol).unwrap();


        let arr = [
            "add",
            "wallet",
            "--alias",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let wallet_input = WalletInput::new(&input, Some(String::from("pass"))).unwrap();
        let wallet_a = Wallet::new(wallet_input).unwrap();
        Wallet::add(&config_file, &wallet_a).unwrap();


        let arr = [
            "add",
            "wallet",
            "--alias",
            "wallet_b",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let wallet_input = WalletInput::new(&input, Some(String::from("pass"))).unwrap();
        let wallet_b = Wallet::new(wallet_input).unwrap();
        Wallet::add(&config_file, &wallet_b).unwrap();


        let sender = AccountKeyring::Alice;

        let recipient = AccountId32::from(wallet_a.public);
        let extrinsic_hash = balance_transfer(sender, &recipient).await.unwrap();
        let event = BalanceTransferEvent;
        event.submit(&extrinsic_hash).await.unwrap();

        let recipient = AccountId32::from(wallet_b.public);
        let extrinsic_hash = balance_transfer(sender, &recipient).await.unwrap();
        let event = BalanceTransferEvent;
        event.submit(&extrinsic_hash).await.unwrap();


        let arr = [
            "add",
            "owner",
            "--account",
            format!("{}", &alice.alias).as_str(),
            "--wallet",
            format!("{}", wallet_a.alias).as_str()
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let owner = Owner::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        owner.add().await.unwrap();


        let arr = [
            "add",
            "owner",
            "--account",
            format!("{}", &bob.alias).as_str(),
            "--wallet",
            format!("{}", wallet_b.alias).as_str()
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let owner = Owner::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        owner.add().await.unwrap();

        config_file
    }


    #[async_std::test]
    async fn send_to_two_recipients() {

        let config_file = init_sending().await;

        let s: String = rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(7)
            .map(char::from)
            .collect();


        let file_name = format!("temp_{}_text.txt", s);
        let home_dir = dirs::home_dir().unwrap();
        let home_path = home_dir.as_path();
        let nolik_dir = home_path.join(".nolik");
        let text_file = nolik_dir.join(file_name.as_str());

        if let Ok(mut file) = fs::File::create(&text_file) {
            let contents = "Hello World";
            file.write_all(contents.as_ref()).unwrap();
        }

        let bob = Account::get(&config_file, String::from("bob")).unwrap();
        let carol = Account::get(&config_file, String::from("carol")).unwrap();

        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--recipient",
            format!("{}", bs58::encode(&carol.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
            "--key",
            "message",
            "--value",
            "test",
            "--file",
            format!("{}", &text_file.clone().into_os_string().into_string().unwrap()).as_str(),
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let bi = BatchInput::new(&mut input, &config_file).unwrap();


        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();
        let ipfs_data = ipfs_file.get().await.unwrap();

        let public_nonce = base64_to_nonce(&ipfs_data.nonce).unwrap();
        let broker = base64_to_public_key(&ipfs_data.broker).unwrap();

        let arr = [
            "send",
            "message",
            "--ipfs-id",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();


        let config = Config::new(&config_file).unwrap();

        for account_output in &config.data.accounts {
            let account = AccountOutput::deserialize(&account_output).unwrap();
            let decrypted_sessions: Vec<Session> = ipfs_data.sessions
                .iter()
                .filter_map(|es| es.decrypt(&public_nonce, &broker, &account.secret).ok())
                .collect();

            let session = decrypted_sessions.first().unwrap();
            let sender = session.group.get_sender();
            let recipients = session.group.get_recipients();

            if sender.ne(&account.public) { continue }

            for pk in recipients {
                let res = ipfs_file.send(&sender, &pk, &ipfs_input.wallet).await.is_ok();

                assert_eq!(res, true);
            }

            break;
        }

        fs::remove_file(config_file.path).unwrap();
        fs::remove_file(text_file).unwrap();
    }


    #[async_std::test]
    async fn send_if_in_blacklist() {

        let config_file = init_sending().await;

        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let bob = Account::get(&config_file, String::from("bob")).unwrap();


        let arr = [
            "update",
            "blacklist",
            "--for",
            format!("{}", bob.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(alice.public).into_string()).as_str(),
            "--wallet",
            "wallet_b"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let blacklist = Blacklist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        blacklist.update().await.unwrap();


        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let bi = BatchInput::new(&mut input, &config_file).unwrap();


        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();
        let ipfs_data = ipfs_file.get().await.unwrap();

        let public_nonce = base64_to_nonce(&ipfs_data.nonce).unwrap();
        let broker = base64_to_public_key(&ipfs_data.broker).unwrap();

        let arr = [
            "send",
            "message",
            "--ipfs-id",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();


        let config = Config::new(&config_file).unwrap();

        for account_output in &config.data.accounts {
            let account = AccountOutput::deserialize(&account_output).unwrap();
            let decrypted_sessions: Vec<Session> = ipfs_data.sessions
                .iter()
                .filter_map(|es| es.decrypt(&public_nonce, &broker, &account.secret).ok())
                .collect();

            let session = decrypted_sessions.first().unwrap();
            let sender = session.group.get_sender();
            let recipients = session.group.get_recipients();

            if sender.ne(&account.public) { continue }

            for pk in recipients {
                let res = ipfs_file.send(&sender, &pk, &ipfs_input.wallet).await.unwrap_err();

                assert_eq!(res, NodeError::PalletAddressInBlacklist);
            }

            break;
        }

        fs::remove_file(config_file.path).unwrap();
    }


    #[async_std::test]
    async fn send_if_in_whitelist() {

        let config_file = init_sending().await;

        let alice = Account::get(&config_file, String::from("alice")).unwrap();
        let bob = Account::get(&config_file, String::from("bob")).unwrap();


        let arr = [
            "update",
            "whitelist",
            "--for",
            format!("{}", bob.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(alice.public).into_string()).as_str(),
            "--wallet",
            "wallet_b"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let whitelist = Whitelist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        whitelist.update().await.unwrap();


        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let bi = BatchInput::new(&mut input, &config_file).unwrap();


        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();
        let ipfs_data = ipfs_file.get().await.unwrap();

        let public_nonce = base64_to_nonce(&ipfs_data.nonce).unwrap();
        let broker = base64_to_public_key(&ipfs_data.broker).unwrap();

        let arr = [
            "send",
            "message",
            "--ipfs-id",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();


        let config = Config::new(&config_file).unwrap();

        for account_output in &config.data.accounts {
            let account = AccountOutput::deserialize(&account_output).unwrap();
            let decrypted_sessions: Vec<Session> = ipfs_data.sessions
                .iter()
                .filter_map(|es| es.decrypt(&public_nonce, &broker, &account.secret).ok())
                .collect();

            let session = decrypted_sessions.first().unwrap();
            let sender = session.group.get_sender();
            let recipients = session.group.get_recipients();

            if sender.ne(&account.public) { continue }

            for pk in recipients {
                let res = ipfs_file.send(&sender, &pk, &ipfs_input.wallet).await.is_ok();

                assert_eq!(res, true);
            }

            break;
        }

        fs::remove_file(config_file.path).unwrap();
    }


    #[async_std::test]
    async fn send_if_not_in_whitelist() {

        let config_file = init_sending().await;

        let bob = Account::get(&config_file, String::from("bob")).unwrap();
        let carol = Account::get(&config_file, String::from("carol")).unwrap();


        let arr = [
            "update",
            "whitelist",
            "--for",
            format!("{}", bob.alias).as_str(),
            "--add",
            format!("{}", bs58::encode(carol.public).into_string()).as_str(),
            "--wallet",
            "wallet_b"
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();

        let whitelist = Whitelist::new(&input, &config_file, Some(String::from("pass"))).unwrap();
        whitelist.update().await.unwrap();


        let arr = [
            "compose",
            "message",
            "--sender",
            "alice",
            "--recipient",
            format!("{}", bs58::encode(&bob.public).into_string()).as_str(),
            "--key",
            "subject",
            "--value",
            "hello",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let mut input = Input::new(args).unwrap();

        let bi = BatchInput::new(&mut input, &config_file).unwrap();


        let secret_nonce = box_::gen_nonce();
        let batch = Batch::new(&bi, &secret_nonce).unwrap();
        let ipfs_file = batch.save().await.unwrap();
        let ipfs_data = ipfs_file.get().await.unwrap();

        let public_nonce = base64_to_nonce(&ipfs_data.nonce).unwrap();
        let broker = base64_to_public_key(&ipfs_data.broker).unwrap();

        let arr = [
            "send",
            "message",
            "--ipfs-id",
            format!("{}", ipfs_file.0).as_str(),
            "--wallet",
            "wallet_a",
        ].map(|el| el.to_string());

        let args = arr.iter();
        let input = Input::new(args).unwrap();
        let ipfs_input = IpfsInput::new(&config_file, &input, Some(String::from("pass"))).unwrap();


        let config = Config::new(&config_file).unwrap();

        for account_output in &config.data.accounts {
            let account = AccountOutput::deserialize(&account_output).unwrap();
            let decrypted_sessions: Vec<Session> = ipfs_data.sessions
                .iter()
                .filter_map(|es| es.decrypt(&public_nonce, &broker, &account.secret).ok())
                .collect();

            let session = decrypted_sessions.first().unwrap();
            let sender = session.group.get_sender();
            let recipients = session.group.get_recipients();

            if sender.ne(&account.public) { continue }

            for pk in recipients {
                let res = ipfs_file.send(&sender, &pk, &ipfs_input.wallet).await.unwrap_err();

                assert_eq!(res, NodeError::PalletAddressNotInWhitelist);
            }

            break;
        }

        fs::remove_file(config_file.path).unwrap();
    }
}