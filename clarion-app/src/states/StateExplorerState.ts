import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { Contract, StacksContractInterface } from "../types";
import { RootState } from "../stores/root";
import { ContractFieldTarget, StateExplorerStateUpdateWatchData, TargetType } from "./NetworkingState";

export interface StateExplorerState {
  initialized: boolean,
  contractsIdentifiers: Array<string>;
  contracts: { [contractIdentifier: string]: StacksContractInterface };
  fields: { [fieldIdentifier: string]: string };
  wallets: Array<string>,
  activeContractIdentifier?: string;
  activeField?: string;
}


const initialState: StateExplorerState = {
  initialized: false,
  contractsIdentifiers: [],
  wallets: [],
  contracts: {},
  fields: {},
  activeContractIdentifier: undefined,
  activeField: undefined,
};

export const stateExplorerSlice = createSlice({
  name: "stateExplorer",
  initialState,
  reducers: {
    activateField: (
      state: StateExplorerState,
      action: PayloadAction<ContractFieldTarget>
    ) => {
      state.activeField = action.payload.field_name;
      state.activeContractIdentifier = action.payload.contract_identifier;
    },
    updateContracts: (
      state: StateExplorerState,
      action: PayloadAction<Array<Contract>>
    ) => {
      state.contractsIdentifiers = [];
      state.contracts = {};
      for (const contract of action.payload) {
        state.contracts[contract.contract_identifier] = contract.interface;
        state.contractsIdentifiers.push(contract.contract_identifier);
      }
    },
    updateField: (
      state: StateExplorerState,
      action: PayloadAction<StateExplorerStateUpdateWatchData>
    ) => {
      if ('Var' in action.payload.field_values) {
        let payload = JSON.stringify(action.payload);
        state.fields[`${action.payload.contract_identifier}::${action.payload.field_name}`] = payload;
      } else if ('Map' in action.payload.field_values) {
        let payload = JSON.stringify(action.payload);
        state.fields[`${action.payload.contract_identifier}::${action.payload.field_name}`] = payload;
      } else if ('Ft' in action.payload.field_values) {
        let payload = JSON.stringify(action.payload);
        state.fields[`${action.payload.contract_identifier}::${action.payload.field_name}`] = payload;
      } else if ('Nft' in action.payload.field_values) {
        let payload = JSON.stringify(action.payload);
        state.fields[`${action.payload.contract_identifier}::${action.payload.field_name}`] = payload;
      } 
    },
  },
});

export const { activateField, updateContracts, updateField } = stateExplorerSlice.actions;

export const selectContracts = (state: RootState) =>
  state.stateExplorer.contracts;

export const selectContractsIdentifiers = (state: RootState) =>
  state.stateExplorer.contractsIdentifiers;

export const selectWallets = (state: RootState) =>
  state.stateExplorer.wallets;

export const selectFields = (state: RootState) =>
  state.stateExplorer.fields;

export const selectActiveContractIdentifier = (state: RootState) =>
  state.stateExplorer.activeContractIdentifier;

export const selectActiveField = (state: RootState) =>
  state.stateExplorer.activeField;

export default stateExplorerSlice.reducer;
