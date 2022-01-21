import {
  Block,
  BlockIdentifier,
  DataVarField,
  DataMapField,
  DataFtField,
  DataNftField
} from "./clarinet";

export enum PollState {
  None = "None",
  Initialization = "StateExplorerInitialization",
  Sync = "StateExplorerSync",
  Active = "StateExplorerActive",
}

export type PollStateData = Record<
  PollState,
  PollStateNone | PollStateInitialization | PollStateSync | PollStateActive
>;
export type PollStateUpdateData = Record<
  PollState,
  PollStateInitializationUpdate | PollStateSyncUpdate | PollStateActiveUpdate
>;

export interface PollStateUpdate {
  update: PollStateUpdateData;
}

export interface PollStateNone {}

export interface PollStateInitialization {
  manifest_path: string;
}

export interface PollStateSync {
  stacks_chain_tip?: BlockIdentifier;
  bitcoin_chain_tip?: BlockIdentifier;
  expected_contracts_identifiers: string[];
}

export interface PollStateActive {
  stacks_chain_tip: BlockIdentifier;
  bitcoin_chain_tip: BlockIdentifier;
  contract_identifier: string;
  field: string;
}

export interface PollStateInitializationUpdate {
  contracts: Contract[];
}

export interface Contract {
  contract_identifier: string;
  interface: StacksContractInterface;
}

export interface PollStateSyncUpdate {
  stacks_chain_tip: Block;
  bitcoin_chain_tip: Block;
  contracts: Contract[];
  expected_contracts_identifiers: string[];
}

export interface PollStateActiveUpdate {
  bitcoin_chain_blocks: Block[];
  stacks_chain_blocks: Block[];
  field_values: any;
}

export enum ContractState {
  Indexing = "Indexing",
  Ready = "Ready",
}

export interface ContractStateIndexing {}

export interface ContractStateReady {
  contract_identifier: string;
  interface: StacksContractInterface;
}

export type ContractStateData = Record<
  ContractState,
  ContractStateIndexing | ContractStateReady
>;

  /**
   * Lorem ipsum
   * @export
   * @interface StacksContractInterface
   */
export interface StacksContractInterface {
    /**
     * List of defined methods
     * @type {Array<object>}
     * @memberof StacksContractInterface
     */
    functions: Array<object>;
    /**
     * List of defined variables
     * @type {Array<object>}
     * @memberof StacksContractInterface
     */
    variables: Array<DataVarField>;
    /**
     * List of defined data-maps
     * @type {Array<object>}
     * @memberof StacksContractInterface
     */
    maps: Array<DataMapField>;
    /**
     * List of fungible tokens in the contract
     * @type {Array<DataFtField>}
     * @memberof StacksContractInterface
     */
    fungible_tokens: Array<DataFtField>;
    /**
     * List of non-fungible tokens in the contract
     * @type {Array<DataNftField>}
     * @memberof StacksContractInterface
     */
    non_fungible_tokens: Array<DataNftField>;
  }
  