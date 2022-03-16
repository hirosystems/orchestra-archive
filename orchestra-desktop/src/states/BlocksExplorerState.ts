import { createSlice, PayloadAction } from "@reduxjs/toolkit";
import { RootState } from "../stores/root";
import { BitcoinBlock, StacksBlock } from "../types/clarinet";

export interface BlocksExplorerState {
  initialized: boolean;
  bitcoinChainTipIndex: number;
  stacksChainTipIndex: number;
  bitcoinBlocks: { [blockIndex: number]: BitcoinBlock };
  stacksBlocks: { [blockIndex: number]: StacksBlock };
}

const initialState: BlocksExplorerState = {
  initialized: false,
  bitcoinChainTipIndex: -1,
  stacksChainTipIndex: -1,
  bitcoinBlocks: {},
  stacksBlocks: {},
};

export const blocksExplorerStateSlice = createSlice({
  name: "blocksExplorer",
  initialState,
  reducers: {
    appendStacksBlocks: (
      state: BlocksExplorerState,
      action: PayloadAction<Array<StacksBlock>>
    ) => {
      for (let block of action.payload) {
        let blockIndex = block.block_identifier.index;
        state.stacksBlocks[blockIndex] = block;
        state.stacksChainTipIndex = blockIndex;
      }
    },
    appendBitcoinBlocks: (
      state: BlocksExplorerState,
      action: PayloadAction<Array<BitcoinBlock>>
    ) => {
      for (let block of action.payload) {
        let blockIndex = block.block_identifier.index;
        state.bitcoinBlocks[blockIndex] = block;
        state.bitcoinChainTipIndex = blockIndex;
      }
    },
  },
});

export const { appendStacksBlocks, appendBitcoinBlocks } =
  blocksExplorerStateSlice.actions;

export const getBitcoinChainTip = (
  state: RootState
): BitcoinBlock | undefined =>
  state.blocksExplorer.bitcoinBlocks[state.blocksExplorer.bitcoinChainTipIndex];

export const getStacksChainTip = (state: RootState): StacksBlock | undefined =>
  state.blocksExplorer.stacksBlocks[state.blocksExplorer.stacksChainTipIndex];

export default blocksExplorerStateSlice.reducer;
