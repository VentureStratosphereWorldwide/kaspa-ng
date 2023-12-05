use crate::imports::*;
use super::*;

pub struct Overview<'manager> {
    context : &'manager mut ManagerContext,
    editor_size : Vec2,
}

impl<'manager> Overview<'manager> {
    pub fn new(context : &'manager mut ManagerContext) -> Self {
        Self { context, editor_size : Vec2::INFINITY }
    }

    pub fn render(&mut self, core: &mut Core, ui : &mut Ui, rc : &RenderContext<'_>) {
        use egui_phosphor::light::{ARROW_CIRCLE_UP,ARROWS_DOWN_UP,QR_CODE};

        core.apply_mobile_style(ui);

        ui.separator();
        ui.add_space(8.);

        egui::ScrollArea::vertical()
            .id_source("overview_metrics")
            .auto_shrink([false; 2])
            .show(ui, |ui| {

                self.editor_size = Vec2::new(ui.available_width() * 0.75, 32.);

                ui.vertical_centered(|ui| {

                    self.render_address(core, ui, rc);
                    
                    self.render_balance(core, ui, rc);

                    if !core.state().is_synced() || !core.state().is_connected() {
                        self.render_network_state(core,ui);
                        return;
                    }

                    if self.context.action.is_sending() {
                        self.render_send_ui(core, ui, rc);
                    } else {
                        
                        self.render_qr(core, ui, rc);

                        ui.vertical_centered(|ui|{
                        
                            ui.add_space(8.);
                            ui.horizontal(|ui| {

                                let mut layout = CenterLayoutBuilder::new();
                                
                                layout = layout.add(Button::new(format!("{} Send", ARROW_CIRCLE_UP)).min_size(theme_style().medium_button_size()), |(this, _):&mut (&mut Overview<'_>, &mut Core)| {
                                    this.context.action = Action::Estimating;
                                    this.context.transaction_kind = Some(TransactionKind::Send);
                                    this.context.focus.next(Focus::Address);
                                });

                                if core.account_collection().as_ref().map(|collection|collection.len()).unwrap_or(0) > 1 {
                                    layout = layout.add(Button::new(format!("{} Transfer", ARROWS_DOWN_UP)).min_size(theme_style().medium_button_size()), |(this,_)| {
                                        this.context.action = Action::Estimating;
                                        this.context.transaction_kind = Some(TransactionKind::Transfer);
                                    });
                                }
                                layout = layout.add(Button::new(format!("{} Request", QR_CODE)).min_size(theme_style().medium_button_size()), |(_,core)| {
                                    core.select::<modules::Request>();

                                });

                                layout.build(ui,&mut (self,core));
                            });
                        });

                    }
                });
            });
    }

    fn render_network_state(&mut self, core : &mut Core, ui: &mut Ui) {
        use egui_phosphor::light::{CLOUD_SLASH,CLOUD_ARROW_DOWN};

        const ICON_SPACING: f32 = 24.0;
        ui.vertical_centered(|ui|{
            // ui.add_space(16.);
            if !core.state().is_connected() {
                ui.add_space(ICON_SPACING);
                ui.label(
                    RichText::new(CLOUD_SLASH)
                        .size(theme_style().icon_size_large)
                        .color(theme_color().icon_color_default)
                );
                ui.add_space(ICON_SPACING);
                
                ui.label("You are currently not connected to the Kaspa node.");
            } else if !core.state().is_synced() {
                
                ui.add_space(ICON_SPACING);
                ui.label(
                    RichText::new(CLOUD_ARROW_DOWN)
                        .size(theme_style().icon_size_large)
                        .color(theme_color().icon_color_default)
                );
                ui.add_space(ICON_SPACING);

                ui.label("The node is currently syncing with the Kaspa p2p network.");
                ui.add_space(16.);
                ui.label("Please wait for the node to sync or connect to a remote node.");
            }
            ui.add_space(32.);
            ui.label("You can configure a remote connection in Settings");
            ui.add_space(16.);
            if ui.large_button("Go to Settings").clicked() {
                core.select::<modules::Settings>();
            }
        });


    }

    fn render_address(&mut self, _core: &mut Core, ui : &mut Ui, rc : &RenderContext<'_>) {
        use egui_phosphor::light::CLIPBOARD_TEXT;
        let address = format_address(rc.context.address(), Some(8));
        if ui.add(Label::new(format!("Address: {address} {CLIPBOARD_TEXT}")).sense(Sense::click()))
            // .on_hover_ui_at_pointer(|ui|{
            //     ui.vertical(|ui|{
            //         ui.add(Label::new(format!("{}", context.address().to_string())));
            //         ui.add_space(16.);
            //         ui.label("Click to copy address to clipboard".to_string());
            //     });
            // })
            .clicked() {
                ui.output_mut(|o| o.copied_text = rc.context.address().to_string());
                runtime().notify(UserNotification::info(format!("{CLIPBOARD_TEXT} {}", i18n("Copied to clipboard"))).short())
            }
    }

    fn render_balance(&mut self, core: &mut Core, ui : &mut Ui, rc: &RenderContext<'_>) {

        // let theme = theme();
        let RenderContext { account, network_type, .. } = rc;

        ui.add_space(10.);

        if let Some(balance) = account.balance() {
            
            if !core.state().is_synced() {
                ui.label(
                    s2kws_layout_job(balance.mature, network_type, theme_color().balance_syncing_color,FontId::proportional(28.))
                );
                ui.label(RichText::new(i18n("The balance may be out of date during node sync")).size(12.).color(theme_color().balance_syncing_color));
                return;
            } else {
                ui.label(
                    s2kws_layout_job(balance.mature, network_type, theme_color().balance_color,FontId::proportional(28.))
                );
            }

            if balance.pending != 0 {
                ui.label(format!(
                    "Pending: {}",
                    sompi_to_kaspa_string_with_suffix(
                        balance.pending,
                        network_type
                    )
                ));
            }
            if balance.outgoing != 0 {
                ui.label(format!(
                    "Sending: {}",
                    sompi_to_kaspa_string_with_suffix(
                        balance.outgoing,
                        network_type
                    )
                ));
            }

            ui.add_space(10.);

            let suffix = if balance.pending_utxo_count != 0 && balance.stasis_utxo_count != 0 {
                format!(" ({} pending, {} processing)", balance.pending_utxo_count, balance.stasis_utxo_count)
            } else if balance.pending_utxo_count != 0 {
                format!(" ({} pending)", balance.pending_utxo_count)
            } else if balance.stasis_utxo_count != 0 {
                format!(" ({} processing)", balance.stasis_utxo_count)
            } else {
                "".to_string()
            };

            if self.context.transaction_kind.is_none() {
                ui.label(format!(
                    "UTXOs: {}{suffix}",
                    balance.mature_utxo_count.separated_string(),
                ));
            }
        } else {
            ui.label("Balance: N/A");
        }



    }

    fn render_qr(&mut self, _core: &mut Core, ui : &mut Ui, rc: &RenderContext<'_>) {
        let RenderContext { context, .. } = rc;

        // let scale = if self.context.action == Action::None { 1. } else { 0.35 };
        ui.add(
            egui::Image::new(ImageSource::Bytes { uri : Cow::Owned(context.uri()), bytes: context.qr() })
            .fit_to_original_size(1.0)
            .texture_options(TextureOptions::NEAREST)
        );
    }

    fn render_transfer_account_selector(&mut self, core: &mut Core, ui: &mut egui::Ui, rc: &RenderContext<'_>) {
        let RenderContext { network_type, .. } = rc;

        let default_account = core.account_collection().as_ref().and_then(|collection|{
            if collection.len() <= 1 {
                unreachable!("expecting least 2 accounts");
            }
            if collection.len() == 2 {
                collection.list().iter().find(|account|account.id() != rc.account.id()).cloned()
            } else {
                None
            }
        });

        if let Some(account) = default_account {
            self.context.transfer_to_account = Some(account.clone());
            ui.label(format!("Transferring funds to: {}", account.name_or_id()));
            ui.label(format!("Destination balance: {}", sompi_to_kaspa_string_with_suffix(account.balance().map(|balance|balance.mature).unwrap_or(0), network_type)));
        } else {

            if self.context.transfer_to_account.as_ref().map(|account|account.id() == rc.account.id()).unwrap_or_default() {
                self.context.transfer_to_account = None;
                self.context.transfer_to_account.take();
            }

            let transfer_to_account = self.context.transfer_to_account.clone();


            PopupPanel::new(ui, "transfer_selector_popup",|ui|{ 
                let response = ui.vertical_centered(|ui| {
                    if let Some(account) = transfer_to_account {
                        let response = ui.add(Label::new(format!("Transferring funds to: {} ⏷", account.name_or_id())).sense(Sense::click()));
                        ui.label(format!("Destination balance: {}", sompi_to_kaspa_string_with_suffix(account.balance().map(|balance|balance.mature).unwrap_or(0), network_type)));
                        response
                    } else {
                        ui.add(Label::new("Please select destination account ⏷").sense(Sense::click()))
                    }
                });

                response.inner
            }, |ui, _| {

                egui::ScrollArea::vertical()
                    .id_source("transfer_selector_popup_scroll")
                    .auto_shrink([true; 2])
                    .show(ui, |ui| {

                        if let Some(account_collection) = core.account_collection() {
                            account_collection.iter().for_each(|account| {
                                if account.id() == rc.account.id() {
                                    return;
                                }

                                if ui.account_selector_button(account, network_type, false).clicked() {
                                    self.context.transfer_to_account = Some(account.clone());
                                }
                            });
                        }

                    });

            })
            .with_min_width(240.)
            .with_close_on_interaction(true)
            .build(ui);
        }
    }

    fn render_address_input(&mut self, _core: &mut Core, ui: &mut egui::Ui, rc: &RenderContext<'_>) {
        let RenderContext { network_type, .. } = rc;


        TextEditor::new(
            &mut self.context.destination_address_string,
            // None,
            &mut self.context.focus,
            Focus::Address,
            |ui, text| {
                ui.add_space(8.);
                ui.label(egui::RichText::new("Enter destination address").size(12.).raised());
                ui.add_sized(self.editor_size, TextEdit::singleline(text)
                    .vertical_align(Align::Center))
            },
        )
        .change(|address| {
            match Address::try_from(address) {
                Ok(address) => {
                    let address_network_type = NetworkType::try_from(address.prefix).expect("prefix to network type");
                    if address_network_type != *network_type {
                        self.context.address_status = AddressStatus::NetworkMismatch(address_network_type);
                    } else {
                        self.context.address_status = AddressStatus::Valid;
                    }
                }
                Err(err) => {
                    self.context.address_status = AddressStatus::Invalid(err.to_string());
                }
            }
        })
        .submit(|_, focus|{
            // *focus = Some(Focus::Amount);
            focus.next(Focus::Amount);
        })
        .build(ui);
        
        match &self.context.address_status {
            AddressStatus::Valid => {},
            AddressStatus::None => {},
            AddressStatus::NetworkMismatch(address_network_type) => {
                ui.label(format!("This address if for the different\nnetwork ({address_network_type})"));
            },
            AddressStatus::Invalid(err) => {
                ui.label(format!("Please enter a valid address\n{err}"));
            }
        }




    }

    fn render_estimation_ui(&mut self, core: &mut Core, ui: &mut egui::Ui, rc: &RenderContext<'_>) -> bool {
        use egui_phosphor::light::{CHECK, X};

        let RenderContext { network_type, .. } = rc;

        let mut request_estimate = self.context.request_estimate.unwrap_or_default();

        match self.context.transaction_kind.as_ref().unwrap() {
            TransactionKind::Send => {
                self.render_address_input(core, ui, rc);
            }
            TransactionKind::Transfer => {
                self.render_transfer_account_selector(core, ui, rc);
            }
        }

        let response = TextEditor::new(
            &mut self.context.send_amount_text,
            &mut self.context.focus,
            Focus::Amount,
            |ui, text| {
                ui.add_space(8.);
                ui.label(egui::RichText::new(format!("Enter {} amount to send", kaspa_suffix(network_type))).size(12.).raised());
                ui.add_sized(self.editor_size, TextEdit::singleline(text)
                    .vertical_align(Align::Center))
            },
        )
        .change(|_| {
            request_estimate = true;
        })
        .build(ui);

        if response.text_edit_submit(ui) {
            if self.context.enable_priority_fees {
                self.context.focus.next(Focus::Fees);
            } else if self.update_user_args() {
                self.context.action = Action::Sending;
                self.context.focus.next(Focus::WalletSecret);
            }
        }

        ui.add_space(8.);
        if ui
            .checkbox(&mut self.context.enable_priority_fees,i18n("Include Priority Fees"))
            // .on_hover_text_at_pointer(i18n("Add priority fees to ensure faster confirmation.\nUseful only if the network is congested."))
            .changed() {
            if self.context.enable_priority_fees {
                self.context.focus.next(Focus::Fees);
            } else {
                self.context.focus.next(Focus::Amount);
            }
        }

        if self.context.enable_priority_fees {
            TextEditor::new(
                &mut self.context.priority_fees_text,
                &mut self.context.focus,
                Focus::Fees,
                |ui, text| {
                    ui.add_space(8.);
                    ui.label(egui::RichText::new("Enter priority fees").size(12.).raised());
                    ui.add_sized(self.editor_size, TextEdit::singleline(text)
                        .vertical_align(Align::Center))
                },
            )
            .change(|_| {
                request_estimate = true;
            })
            .submit(|_,_|{
                self.context.action = Action::Sending;
            })
            .build(ui); 
        }

        ui.add_space(8.);
        let ready_to_send = match &*self.context.estimate.lock().unwrap() {
            EstimatorStatus::GeneratorSummary(estimate) => {
                if let Some(final_transaction_amount) = estimate.final_transaction_amount {
                    ui.label(format!("Final Amount: {}", sompi_to_kaspa_string_with_suffix(final_transaction_amount + estimate.aggregated_fees, network_type)));
                }
                let fee_title = if self.context.priority_fees_sompi != 0 {
                    "Network and Priority Fees:"
                } else {
                    "Network Fees:"
                };
                ui.label(format!("{} {}", fee_title, sompi_to_kaspa_string_with_suffix(estimate.aggregated_fees, network_type)));
                ui.label(format!("Transactions: {} UTXOs: {}", estimate.number_of_generated_transactions, estimate.aggregated_utxos));
                
                self.context.address_status == AddressStatus::Valid || self.context.transaction_kind == Some(TransactionKind::Transfer)
            }
            EstimatorStatus::Error(error) => {
                ui.label(RichText::new(error.to_string()).color(theme_color().error_color));
                false
            }
            EstimatorStatus::None => {
                ui.label("Please enter KAS amount to send");
                false
            }
        };
        ui.add_space(8.);

        ui.horizontal(|ui| {
            ui.vertical_centered(|ui|{
                ui.horizontal(|ui| {
                    CenterLayoutBuilder::new()
                        .add_enabled(ready_to_send, Button::new(format!("{CHECK} Send")).min_size(theme_style().medium_button_size()), |this: &mut Overview<'_>| {
                            this.context.action = Action::Sending;
                            this.context.focus.next(Focus::WalletSecret);
                        })
                        .add(Button::new(format!("{X} Cancel")).min_size(theme_style().medium_button_size()), |this| {
                            this.context.reset_send_state();
                        })
                        .build(ui, self)
                });
            });

        });

        request_estimate
    }

    fn render_passphrase_ui(&mut self, _core: &mut Core, ui: &mut egui::Ui, rc: &RenderContext<'_>) -> bool {
        use egui_phosphor::light::{CHECK, X};

        let RenderContext { account, .. } = rc;

        let requires_payment_passphrase = account.requires_bip39_passphrase();
        let mut proceed_with_send = false;

        let response = TextEditor::new(
            &mut self.context.wallet_secret,
            &mut self.context.focus,
            Focus::WalletSecret,
            |ui, text| {
                ui.add_space(8.);
                ui.label(egui::RichText::new("Enter wallet password").size(12.).raised());
                ui.add_sized(self.editor_size, TextEdit::singleline(text)
                    .password(true)
                    .vertical_align(Align::Center))
            },
        )
        .build(ui);

        if response.text_edit_submit(ui) {
            if account.requires_bip39_passphrase() {
                self.context.focus.next(Focus::PaymentSecret);
            } else if !self.context.wallet_secret.is_empty() {
                proceed_with_send = true;
            }
        }

        if requires_payment_passphrase {
            let response = TextEditor::new(
                &mut self.context.payment_secret,
                &mut self.context.focus,
                Focus::PaymentSecret,
                |ui, text| {
                    ui.add_space(8.);
                    ui.label(egui::RichText::new("Enter bip39 passphrase").size(12.).raised());
                    ui.add_sized(self.editor_size, TextEdit::singleline(text)
                        .password(true)
                        .vertical_align(Align::Center))
                },
            )
            .build(ui);

            if response.text_edit_submit(ui) && !self.context.wallet_secret.is_empty() && !self.context.payment_secret.is_empty() {
                proceed_with_send = true;
            }
    
        }

        let is_ready_to_send = !(self.context.wallet_secret.is_empty() || requires_payment_passphrase && self.context.payment_secret.is_empty());

        ui.add_space(8.);
        CenterLayoutBuilder::new()
            .add_enabled(is_ready_to_send, Button::new(format!("{CHECK} Submit")).min_size(theme_style().medium_button_size()), |_this: &mut Overview<'_>| {
                proceed_with_send = true;
            })
            .add(Button::new(format!("{X} Cancel")).min_size(theme_style().medium_button_size()), |this| {
                this.context.action = Action::Estimating;
                this.context.focus.next(Focus::Amount);
            })
            .build(ui,self);



        proceed_with_send
    }

    fn render_send_ui(&mut self, core: &mut Core, ui: &mut egui::Ui, rc: &RenderContext<'_>) {

        let RenderContext { account, network_type, .. } = rc;

        ui.add_space(8.);
        match self.context.transaction_kind.as_ref().unwrap() {
            TransactionKind::Send => {
                ui.label("Sending funds");
                ui.add_space(8.);
            }
            TransactionKind::Transfer => {
                // ui.label("Transferring funds");
            }
        }
        // ui.label("Sending funds");


        let send_result = Payload::<Result<GeneratorSummary>>::new("send_result");


        match self.context.action {
            Action::Estimating => {

                let request_estimate = self.render_estimation_ui(core, ui, rc);

                if request_estimate && self.update_user_args() {

                    let priority_fees_sompi = if self.context.enable_priority_fees {
                        self.context.priority_fees_sompi
                    } else { 0 };

                    let address = match network_type {
                        NetworkType::Testnet => Address::try_from("kaspatest:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqhqrxplya").unwrap(),
                        NetworkType::Mainnet => Address::try_from("kaspa:qqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqqkx9awp4e").unwrap(),
                        _ => panic!("Unsupported network"),
                    };

                    let account_id = account.id();
                    let payment_output = PaymentOutput {
                        address,
                        amount: self.context.send_amount_sompi,
                    };

                    let status = self.context.estimate.clone();
                    spawn(async move {
                        let request = AccountsEstimateRequest {
                            task_id: None,
                            account_id,
                            destination: payment_output.into(),
                            priority_fee_sompi: Fees::SenderPaysAll(priority_fees_sompi),
                            payload: None,
                        };

                        match runtime().wallet().accounts_estimate_call(request).await {
                            Ok(response) => {
                                *status.lock().unwrap() = EstimatorStatus::GeneratorSummary(response.generator_summary);
                            }
                            Err(error) => {
                                *status.lock().unwrap() = EstimatorStatus::Error(error.to_string());
                            }    
                        }

                        runtime().egui_ctx().request_repaint();
                        Ok(())
                    });
                } 

            }

            Action::Sending => {

                let proceed_with_send = self.render_passphrase_ui(core, ui, rc);

                if proceed_with_send {

                    if self.context.destination_address_string.is_not_empty() && self.context.transfer_to_account.is_some() {
                        unreachable!("expecting only one of destination address or transfer to account");
                    }

                    let priority_fees_sompi = if self.context.enable_priority_fees {
                        self.context.priority_fees_sompi
                    } else { 0 };

                    let wallet_secret = Secret::try_from(self.context.wallet_secret.clone()).expect("expecting wallet secret");
                    let payment_secret = account.requires_bip39_passphrase().then_some(Secret::try_from(self.context.payment_secret.clone()).expect("expecting payment secret"));

                    match self.context.transaction_kind.unwrap() {
                        TransactionKind::Send => {

                            let address = Address::try_from(self.context.destination_address_string.as_str()).expect("invalid address");
                            let account_id = account.id();
                            let payment_output = PaymentOutput {
                                address,
                                amount: self.context.send_amount_sompi,
                            };
        
                            spawn_with_result(&send_result, async move {
                                let request = AccountsSendRequest {
                                    account_id,
                                    destination: payment_output.into(),
                                    wallet_secret,
                                    payment_secret,
                                    priority_fee_sompi: Fees::SenderPaysAll(priority_fees_sompi),
                                    payload: None,
                                };
        
                                let generator_summary = runtime().wallet().accounts_send_call(request).await?.generator_summary;
                                runtime().request_repaint();
                                Ok(generator_summary)
                            });

                        }

                        TransactionKind::Transfer => {
                            let destination_account_id = self.context.transfer_to_account.as_ref().unwrap().id();
                            let source_account_id = account.id();
                            let transfer_amount_sompi = self.context.send_amount_sompi;

                            spawn_with_result(&send_result, async move {
                                let request = AccountsTransferRequest {
                                    source_account_id,
                                    destination_account_id,
                                    wallet_secret,
                                    payment_secret,
                                    priority_fee_sompi: Some(Fees::SenderPaysAll(priority_fees_sompi)),
                                    transfer_amount_sompi,
                                };
        
                                let generator_summary = runtime().wallet().accounts_transfer_call(request).await?.generator_summary;
                                runtime().request_repaint();
                                Ok(generator_summary)
                            });
                        }
                    }
            
                    self.context.action = Action::Processing;
                }

            }
            Action::Processing => {
                ui.add_space(16.);
                ui.add(egui::Spinner::new().size(92.));

                if let Some(result) = send_result.take() {

                    if result.is_ok() {
                        self.context.reset_send_state();
                    }

                    // - TODO - SET AND DISPLAY AN ERROR
                    // - PRESENT CLOSE BUTTON BEFORE CONTINUING
                    // - RESET STATE TO ESTIMATING?

                    self.context.action = Action::None;
                }
            }
            Action::None => {}
        }

    }

    fn update_user_args(&mut self) -> bool {
        let mut valid = true;

        match try_kaspa_str_to_sompi(self.context.send_amount_text.as_str()) {
            Ok(Some(sompi)) => {
                self.context.send_amount_sompi = sompi;
            }
            Ok(None) => {
                self.user_error("Please enter an amount".to_string());
                valid = false;
            }
            Err(err) => {
                self.user_error(format!("Invalid amount: {err}"));
                valid = false;
            }
        }

        match try_kaspa_str_to_sompi(self.context.priority_fees_text.as_str()) {
            Ok(Some(sompi)) => {
                self.context.priority_fees_sompi = sompi;
            }
            Ok(None) => {
                self.context.priority_fees_sompi = 0;
            }
            Err(err) => {
                self.user_error(format!("Invalid fee amount: {err}"));
                valid = false;
            }
        }

        valid
    }

    fn user_error(&self, error : impl Into<String>) {
        *self.context.estimate.lock().unwrap() = EstimatorStatus::Error(error.into());
    }
        
}