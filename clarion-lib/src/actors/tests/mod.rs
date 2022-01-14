
#[cfg(test)]
mod test {

    use std::time::SystemTime;
    use opentelemetry::{KeyValue};
    use opentelemetry::trace::{StatusCode, Span, SpanContext};
    use clarinet_lib::types::{BlockIdentifier, StacksBlockMetadata, StacksBlockData, StacksTransactionData, TransactionIdentifier, StacksTransactionMetadata, StacksTransactionReceipt};
    use std::collections::HashSet;

    use crate::actors::BlockStoreManager;

    
    #[derive(Debug)]
    struct MockedSpan {
        context: SpanContext
    }

    impl MockedSpan {
        pub fn new() -> MockedSpan {
            MockedSpan {
                context: SpanContext::empty_context(),
            }
        }
    }

    impl Span for MockedSpan {
        fn add_event_with_timestamp(
            &mut self,
            _name: String,
            _timestamp: SystemTime,
            _attributes: Vec<KeyValue>,
        ) {}
        fn span_context(&self) -> &SpanContext {
            return &self.context
        }
        fn is_recording(&self) -> bool { true }
        fn set_attribute(&mut self, _attribute: KeyValue) {}
        fn set_status(&mut self, _code: StatusCode, _message: String) {}
        fn update_name(&mut self, _new_name: String) {}
        fn end(&mut self) {}
        fn end_with_timestamp(&mut self, _timestamp: SystemTime) {}
    }

    fn transaction_contract_call_impacting_contract_id(contract_id: String, success: bool) -> StacksTransactionData {
        let mut mutated_contracts_radius = HashSet::new();
        mutated_contracts_radius.insert(contract_id);
        StacksTransactionData {
            transaction_identifier: TransactionIdentifier {
                hash: "0".into()
            },
            operations: vec![],
            raw_tx: "0x00".to_string(),
            execution_cost: None,
            sender: "".into(),
            fee: 0,
            sponsor: None,
            kind: StacksTransactionKind::ContractCall,
            metadata: StacksTransactionMetadata {
                success,
                result: "".into(),
                receipt: StacksTransactionReceipt {
                    mutated_contracts_radius,
                    mutated_assets_radius: HashSet::new(),
                    events: vec![],
                },
                description: "".into(),
            }
        }
    }

    fn block_with_transactions(transactions: Vec<StacksTransactionData>) -> StacksBlockData {
        StacksBlockData {
            block_identifier: BlockIdentifier { index: 1, hash: "1".into() },
            parent_block_identifier: BlockIdentifier { index: 0, hash: "0".into() },
            timestamp: 0,
            transactions,
            metadata: StacksBlockMetadata { 
                bitcoin_anchor_block_identifier: BlockIdentifier { index: 0, hash: "0".into() }, 
                pox_cycle_index: 0, 
                pox_cycle_position: 0,
                pox_cycle_length: 0 
            }
        }
    }

}