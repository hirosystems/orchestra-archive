import './App.css';
import { useRootSelector, useRootDispatch } from "./hooks/useRootSelector";
import styled from "styled-components";
import StateExplorer from './pages/StateExplorer'
import { ChainView } from './components/Chain/ChainView';
import { initiateBootSequence, selectManifestFileWatched, selectProtocolData, selectProtocolName } from './states/NetworkingState';
import { listen } from '@tauri-apps/api/event'
import { VersionsIcon, CodeIcon } from '@primer/octicons-react'
import { StyledOcticon } from '@primer/react'
import SelectManifest from './pages/SelectManifest'
import LoadingProtocol from './pages/LoadingProtocol'


export const CentringContainer = styled.div`
    display: flex;
    flex-direction: row;
    height: 100%;
    justify-content: center;
`

export const VerticalContainer = styled.div`
    position: absolute;
    display: flex;
    flex-direction: column; 
    align-items: stretch;
    right: 0;
    bottom: 0;
    left: 0;
    height: 100%;
`

export const TopContainer = styled.div`
    display: flex;
    flex-direction: row;
    z-index: 0;
    overflow-x: hidden;
    overflow-y: auto; 
    -ms-overflow-style: none;
`

export const BottomContainer = styled.div`
    background-color: #181818;
    height: 74px;
    width: 100%;
    border-top: 1px solid #282828;
    position: absolute;
    bottom: 0;
    z-index: 1;
`

export const Menu = styled.div`
    background-color: #000000;
    width: 72px;
    padding-top: 64px;
    padding-left: 24px;
    display: flex;
    flex-direction: column;
    gap: 40px;
    position: absolute;
    left: 0;
    top: 0;
    bottom 74px;
`

export const MainCanvas = styled.div`
    padding-top: 32px;
    padding-left: 72px;
    -ms-overflow-style: none;
`

export const RightPanel = styled.div`
    background-color: rgba(0, 0, 0, 0.8);
    width: 300px;
    font-weight: 600;
    padding-top: 32px;
    padding-left: 16px;
    font-size: 14px;
    color: #FFFFFF;
    position: absolute;
    right: 0;
    top: 0;
    bottom: 74px;
    -ms-overflow-style: none;
`

export const Feature = styled.div`
    text-transform: uppercase;
    text-align: left;
    font-size: 20px;
    font-weight: 700;
    padding: 6px;
    padding-right: 12px;
    padding-top: 0px;
    letter-space: 0.03em;
    cursor: default;
    color: #FFFFFF;
`

function Content() {
    const manifestFile = useRootSelector(selectManifestFileWatched);
    const protocolData = useRootSelector(selectProtocolData);
    let dispatch = useRootDispatch();

    let subDom = (  
        <CentringContainer data-tauri-drag-region>
            <SelectManifest data-tauri-drag-region></SelectManifest>
        </CentringContainer>
    );

    if (manifestFile !== undefined) {
        if (protocolData === undefined) {
            subDom = (
                <CentringContainer data-tauri-drag-region>
                    <LoadingProtocol data-tauri-drag-region></LoadingProtocol>
                </CentringContainer>
            );
        } else {
            subDom = (
                <TopContainer>
                    <Menu data-tauri-drag-region>
                        <StyledOcticon icon={VersionsIcon} size={24} sx={{ color: 'fg.onEmphasis' }} />
                        <StyledOcticon icon={CodeIcon} size={24} sx={{ color: 'fg.subtle' }} />
                    </Menu>
                    <MainCanvas data-tauri-drag-region>
                        <StateExplorer />
                    </MainCanvas>
                    <RightPanel data-tauri-drag-region>Protocol Activity</RightPanel>
                </TopContainer>
            );
        }
    } else {
        listen('tauri://file-drop', (event: any) => {
            let manifestPath = event.payload[0];
            dispatch(initiateBootSequence(manifestPath));
        })
    }

    return (
        <VerticalContainer>
            <style>{'body { background-color: #121212; }'}</style>
            {subDom}
            <BottomContainer>
                <ChainView />
            </BottomContainer>
        </VerticalContainer>
    );
}

export default Content;
