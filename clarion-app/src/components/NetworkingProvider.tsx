import { useEffect, useCallback, createContext, ReactChild } from "react";
import { updateContracts } from "../states/StateExplorerState";
import { useInterval } from '../hooks';
import { selectRequest, selectShouldPoll } from '../states/NetworkingState';
import { useRootSelector } from "../hooks/useRootSelector";
import { PollStateUpdate, ContractStateReady } from "../types";

const ws = new WebSocket("ws://127.0.0.1:2404");

const SocketContext = createContext(ws);

interface ISocketProvider {
    children: ReactChild;
}

const NetworkingProvider = (props: ISocketProvider) => {

    const request = useRootSelector(selectRequest);
    const shouldPoll = useRootSelector(selectShouldPoll);

    useInterval(
        () => {
            ws.send(JSON.stringify(request));
        }, shouldPoll ? 5000 : null);

    const onMessage = useCallback((message) => {
        const data: PollStateUpdate = JSON.parse(message?.data);
        if (data.update.Initialization) {
            let value: any = { ...data.update.Initialization };
            let contracts: Array<ContractStateReady> = value.contracts;
            updateContracts(contracts);
        }
    }, []);

    useEffect(() => {
        ws.addEventListener("message", onMessage);
        return () => {
            ws.removeEventListener("message", onMessage);
        };
    }, [onMessage]);

    return (
        <SocketContext.Provider value={ws}>{props.children}</SocketContext.Provider>
    );
}

export { SocketContext, NetworkingProvider };