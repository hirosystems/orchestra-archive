import { useEffect, useCallback, createContext, ReactChild } from "react";
import { updateContracts, updateField } from "../states/StateExplorerState";
import { useInterval } from '../hooks';
import { selectNextRequest, StateExplorerStateUpdate, updateBootSequence, updateBlockIdentifierForContractField, buildNextRequest } from '../states/NetworkingState';
import { useRootSelector, useRootDispatch } from "../hooks/useRootSelector";
import { Contract } from "../types";
import { appendBitcoinBlocks, appendStacksBlocks } from "../states/BlocksExplorerState";

const WS_ADDRESS = "ws://127.0.0.1:2404";
const WS_POLL_INTERVAL = 5000;

const ws = new WebSocket(WS_ADDRESS);

const SocketContext = createContext(ws);

interface ISocketProvider {
    children: ReactChild;
}

const NetworkingProvider = (props: ISocketProvider) => {

    let dispatch = useRootDispatch();
    let nextRequest = useRootSelector(selectNextRequest);

    const performRequest = () => {
        if (nextRequest !== undefined) {
            ws.send(JSON.stringify(nextRequest));
        }
    };

    const onMessage = useCallback((message) => {
        const data: StateExplorerStateUpdate = JSON.parse(message?.data);
        if ('BootNetwork' in data.update) {
            let payload = {...data.update.BootNetwork};
            if (payload.contracts.length > 0) {
                dispatch(updateContracts(payload.contracts));
            }
            dispatch(updateBootSequence(payload));
        } else if ('StateExplorerWatch' in data.update) {
            let payload = {...data.update.StateExplorerWatch};
            if (payload.stacks_blocks.length > 0) {
                let fieldIdentifier = `${payload.contract_identifier}::${payload.field_name}`;
                let block = payload.stacks_blocks[payload.stacks_blocks.length - 1];
                dispatch(updateBlockIdentifierForContractField([fieldIdentifier, block.block_identifier]));
            }
            dispatch(updateField(payload));
            dispatch(appendStacksBlocks(payload.stacks_blocks));
            dispatch(appendBitcoinBlocks(payload.bitcoin_blocks));
        }
    }, [dispatch]);

    useEffect(() => {
        ws.addEventListener("message", onMessage);
        return () => {
            ws.removeEventListener("message", onMessage);
        };
    }, [onMessage]);

    useEffect(() => performRequest());

    useInterval(() => {
        dispatch(buildNextRequest(1));
    }, WS_POLL_INTERVAL);

    return (
        <SocketContext.Provider value={ws}>{props.children}</SocketContext.Provider>
    );
}

export { SocketContext, NetworkingProvider };