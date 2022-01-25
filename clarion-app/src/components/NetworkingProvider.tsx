import { useEffect, useCallback, createContext, ReactChild } from "react";
import { updateContracts, updateField } from "../states/StateExplorerState";
import { useInterval } from '../hooks';
import { selectRequestQueue, StateExplorerStateUpdate, StateExplorerStateUpdateInit, StateExplorerStateUpdateInitData, StateExplorerStateUpdateWatchData, dequeueRequest, StateExplorerStateUpdateWatch } from '../states/NetworkingState';
import { useRootSelector, useRootDispatch } from "../hooks/useRootSelector";
import { Contract } from "../types";

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

    useInterval(
        () => {
            if (requestQueue.nextRequest) {
                let req = JSON.stringify(requestQueue.nextRequest);
                ws.send(req);
                dispatch(dequeueRequest(requestQueue.nextRequest));
            }
    }, WS_POLL_INTERVAL);


    const onMessage = useCallback((message) => {
        const data: StateExplorerStateUpdate = JSON.parse(message?.data);
        if ('StateExplorerInitialization' in data.update) {
            let payload = {...data.update.StateExplorerInitialization};
            let contracts: Array<Contract> = payload.contracts;
            dispatch(updateContracts(contracts));
        } else if ('StateExplorerWatch' in data.update) {
            let payload = {...data.update.StateExplorerWatch};
            dispatch(updateField(payload));
        }
    }, [dispatch]);

    useEffect(() => {
        ws.addEventListener("message", onMessage);
        return () => {
            ws.removeEventListener("message", onMessage);
        };
    }, [onMessage]);

    // useEffect(() => {
    //     if (requestQueue.nextRequest) {
    //         let req = JSON.stringify(requestQueue.nextRequest);
    //         alert(req);
    //         ws.send(req);
    //         dispatch(dequeueRequest(requestQueue.nextRequest));
    //     }
        // let timer1 = setTimeout(() => {
        //     if (requestQueue.nextRequest) {            
        //         ws.send(JSON.stringify(requestQueue.nextRequest));
        //         dispatch(dequeueRequest(requestQueue.nextRequest));
        //     }
        // }, WS_POLL_INTERVAL);
        // return () => {
        //     clearTimeout(timer1);
        // };
    // });

    return (
        <SocketContext.Provider value={ws}>{props.children}</SocketContext.Provider>
    );
}

export { SocketContext, NetworkingProvider };