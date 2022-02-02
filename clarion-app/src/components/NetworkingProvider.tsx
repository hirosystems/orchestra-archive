import { useEffect, useCallback, createContext, ReactChild } from "react";
import { updateContracts, updateField } from "../states/StateExplorerState";
import { useInterval } from '../hooks';
import { selectIsNetworkBooting, selectRequestQueue, StateExplorerStateUpdate, updateBootSequence, dequeueRequest, initializeStateExplorer } from '../states/NetworkingState';
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
    let requestQueue = useRootSelector(selectRequestQueue);
    let isNetworkBooting = useRootSelector(selectIsNetworkBooting);

    const performRequest = () => {
        if (requestQueue.nextRequest) {
            let req = JSON.stringify(requestQueue.nextRequest);
            ws.send(req);
            dispatch(dequeueRequest(requestQueue.nextRequest));
        }
    };

    const onMessage = useCallback((message) => {
        const data: StateExplorerStateUpdate = JSON.parse(message?.data);
        if ('BootNetwork' in data.update) {
            let payload = {...data.update.BootNetwork};
            if (payload.protocol_deployed) {
                dispatch(initializeStateExplorer);
            }
            if (payload.contracts.length > 0) {
                dispatch(updateContracts(payload.contracts));
            }
            dispatch(updateBootSequence(payload));
        } else if ('StateExplorerInitialization' in data.update) {
            let payload = {...data.update.StateExplorerInitialization};
            let contracts: Array<Contract> = payload.contracts;
            dispatch(updateContracts(contracts));
        } else if ('StateExplorerWatch' in data.update) {
            let payload = {...data.update.StateExplorerWatch};
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
        if (!isNetworkBooting) {
            performRequest()
        } 
    }, WS_POLL_INTERVAL);

    return (
        <SocketContext.Provider value={ws}>{props.children}</SocketContext.Provider>
    );
}

export { SocketContext, NetworkingProvider };