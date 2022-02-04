import './App.css';
import { useRootSelector, useRootDispatch } from "./hooks/useRootSelector";
import styled from "styled-components";
import StateExplorer from './pages/StateExplorer'
import SelectManifest from './pages/SelectManifest'
import { Chain } from './components/Chain';
import { initiateBootSequence, selectManifestFileWatched, selectProtocolName } from './states/NetworkingState';
import { listen } from '@tauri-apps/api/event'

export const Header = styled.div`
    height: 100px;
    display: flex;
    flex-flow: row wrap;
    cursor: default;
    justify-content: flex-start;
    gap: 48px;
    // background-color: rgba(240, 240, 240, 0.7);
`

export const ProtocolOverview = styled.div`
  width: 256px;
  padding-top: 30px;
  padding-left: 16px;
  color: rgba(0, 0, 0, 0.7);
  -webkit-user-select: none;      
  -moz-user-select: none;
  -ms-user-select: none;
  user-select: none;
  cursor: default;

`

export const ProtocolLegend = styled.div`
  text-transform: uppercase;
  font-size: 11.5px;
  font-weight: 600;
  padding-left: 6px;
  padding-bottom: 0px;
  margin-top: -10px;
  letter-space: 0.03em;
`

export const ProtocolName = styled.div`
    text-transform: uppercase;
    text-align: left;
    font-size: 20px;
    font-weight: 700;
    padding: 6px;
    padding-right: 12px;
    padding-top: 0px;
    letter-space: 0.03em;
    cursor: default;
`

function Content() {
    const manifestFile = useRootSelector(selectManifestFileWatched);
    const protocolName = useRootSelector(selectProtocolName);

    let dispatch = useRootDispatch();


    let subDom = (<SelectManifest data-tauri-drag-region></SelectManifest>);
    if (manifestFile !== undefined) {
        subDom = (
            <div>
                <Header data-tauri-drag-region>
                    <ProtocolOverview data-tauri-drag-region>
                        <ProtocolName data-tauri-drag-region>{protocolName}</ProtocolName>
                        <ProtocolLegend data-tauri-drag-region>Protocol</ProtocolLegend>
                        {/* <HiroIcon/> */}
                    </ProtocolOverview>
                    <Chain />
                </Header>
                <StateExplorer />
            </div>
        );
    } else {
        listen('tauri://file-drop', (event: any) => {
            let manifestPath = event.payload[0];
            dispatch(initiateBootSequence(manifestPath));
        })
    }

    return subDom;
}

export default Content;
