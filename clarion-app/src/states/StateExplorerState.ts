import { Contract, StacksContractInterface } from "../types";
import { createSlice, PayloadAction } from '@reduxjs/toolkit';

export interface StateExplorerState {
    expectedContractsIdentifiers: Array<string>;
    contracts: Map<string, StacksContractInterface>;
    activeContractIdentifier?: string;
    activeField?: string;
  }
  
  const initialState: StateExplorerState = {
    expectedContractsIdentifiers: [],
    contracts: new Map(),
    activeContractIdentifier: undefined,
    activeField: undefined,
  };
  
  export const stateExplorerSlice = createSlice({
    name: 'stateExplorer',
    initialState,
    reducers: {
      activateField: (state: StateExplorerState, action: PayloadAction<string>) => {
        state.activeField = action.payload;
      },
      updateContracts: (state: StateExplorerState, action: PayloadAction<Array<Contract>>) => {
        state.expectedContractsIdentifiers = [];
        state.contracts = new Map();
        for (const contract of action.payload) {
            state.contracts.set(contract.contract_identifier, contract.interface);
            state.expectedContractsIdentifiers.push(contract.contract_identifier);
        }
      },
    },
  });
  
  export const { activateField, updateContracts } = stateExplorerSlice.actions;

  export default stateExplorerSlice.reducer;
