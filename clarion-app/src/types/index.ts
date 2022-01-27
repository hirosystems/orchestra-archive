import {
  Block,
  BlockIdentifier,
  DataVarField,
  DataMapField,
  DataFtField,
  DataNftField
} from "./clarinet";

// export enum PollState {
//   None = "None",
//   StateExplorerInitialization = "StateExplorerInitialization",
//   StateExplorerSync = "StateExplorerSync",
//   StateExplorerWatch = "StateExplorerWatch",
// }

// export type PollStateData = Record<
//   PollState,
//   PollStateNone | PollStateInitialization | PollStateSync | PollStateActive
// >;
// export type PollStateUpdateData = Record<
//   PollState,
//   PollStateInitializationUpdate | PollStateSyncUpdate | PollStateActiveUpdate
// >;

// export interface PollStateUpdate {
//   update: PollStateUpdateData;
// }

// export interface PollStateNone {}

// export interface PollStateInitialization {
//   manifest_path: string;
// }

// export interface PollStateSync {
//   stacks_chain_tip?: BlockIdentifier;
//   bitcoin_chain_tip?: BlockIdentifier;
//   expected_contracts_identifiers: string[];
// }

// export interface PollStateActive {
//   stacks_chain_tip: BlockIdentifier;
//   bitcoin_chain_tip: BlockIdentifier;
//   contract_identifier: string;
//   field: string;
// }

// export interface PollStateInitializationUpdate {
//   contracts: Contract[];
// }


// export interface PollStateSyncUpdate {
//   stacks_chain_tip: Block;
//   bitcoin_chain_tip: Block;
//   contracts: Contract[];
//   expected_contracts_identifiers: string[];
// }

// export interface PollStateActiveUpdate {
//   bitcoin_chain_blocks: Block[];
//   stacks_chain_blocks: Block[];
//   field_values: any;
// }

// export enum ContractState {
//   Indexing = "Indexing",
//   Ready = "Ready",
// }

// export interface ContractStateIndexing {}

// export interface ContractStateReady {
//   contract_identifier: string;
//   interface: StacksContractInterface;
// }

// export type ContractStateData = Record<
//   ContractState,
//   ContractStateIndexing | ContractStateReady
// >;

export type ClarityAbiTypeBuffer = { buffer: { length: number } };
export type ClarityAbiTypeStringAscii = { 'string-ascii': { length: number } };
export type ClarityAbiTypeStringUtf8 = { 'string-utf8': { length: number } };
export type ClarityAbiTypeResponse = { response: { ok: ClarityAbiType; error: ClarityAbiType } };
export type ClarityAbiTypeOptional = { optional: ClarityAbiType };
export type ClarityAbiTypeTuple = { tuple: { name: string; type: ClarityAbiType }[] };
export type ClarityAbiTypeList = { list: { type: ClarityAbiType; length: number } };

export type ClarityAbiTypeUInt128 = 'uint128';
export type ClarityAbiTypeInt128 = 'int128';
export type ClarityAbiTypeBool = 'bool';
export type ClarityAbiTypePrincipal = 'principal';
export type ClarityAbiTypeTraitReference = 'trait_reference';
export type ClarityAbiTypeNone = 'none';

export type ClarityAbiTypePrimitive =
  | ClarityAbiTypeUInt128
  | ClarityAbiTypeInt128
  | ClarityAbiTypeBool
  | ClarityAbiTypePrincipal
  | ClarityAbiTypeTraitReference
  | ClarityAbiTypeNone;

export type ClarityAbiType =
  | ClarityAbiTypePrimitive
  | ClarityAbiTypeBuffer
  | ClarityAbiTypeResponse
  | ClarityAbiTypeOptional
  | ClarityAbiTypeTuple
  | ClarityAbiTypeList
  | ClarityAbiTypeStringAscii
  | ClarityAbiTypeStringUtf8
  | ClarityAbiTypeTraitReference;

export const isClarityAbiPrimitive = (val: ClarityAbiType): val is ClarityAbiTypePrimitive =>
  typeof val === 'string';
export const isClarityAbiBuffer = (val: ClarityAbiType): val is ClarityAbiTypeBuffer =>
  (val as ClarityAbiTypeBuffer).buffer !== undefined;
export const isClarityAbiStringAscii = (val: ClarityAbiType): val is ClarityAbiTypeStringAscii =>
  (val as ClarityAbiTypeStringAscii)['string-ascii'] !== undefined;
export const isClarityAbiStringUtf8 = (val: ClarityAbiType): val is ClarityAbiTypeStringUtf8 =>
  (val as ClarityAbiTypeStringUtf8)['string-utf8'] !== undefined;
export const isClarityAbiResponse = (val: ClarityAbiType): val is ClarityAbiTypeResponse =>
  (val as ClarityAbiTypeResponse).response !== undefined;
export const isClarityAbiOptional = (val: ClarityAbiType): val is ClarityAbiTypeOptional =>
  (val as ClarityAbiTypeOptional).optional !== undefined;
export const isClarityAbiTuple = (val: ClarityAbiType): val is ClarityAbiTypeTuple =>
  (val as ClarityAbiTypeTuple).tuple !== undefined;
export const isClarityAbiList = (val: ClarityAbiType): val is ClarityAbiTypeList =>
  (val as ClarityAbiTypeList).list !== undefined;

export interface Contract {
  contract_identifier: string;
  interface: StacksContractInterface;
}

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
  