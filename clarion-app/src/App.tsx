import React from 'react'
import './App.css';
import HiroIcon from './hiro.svg'
import StateExplorer from './pages/StateExplorer'
import {ThemeProvider, BaseStyles, Box, Text, UnderlineNav, StyledOcticon} from '@primer/react'
import {VersionsIcon, DatabaseIcon, TerminalIcon, ZapIcon } from '@primer/octicons-react'
import {NetworkingProvider} from './components/NetworkingProvider';
import { Provider } from 'react-redux'
import { rootStore } from './stores/root'
import styled from "styled-components";
import { Chain } from './components/Chain';

export const Header = styled.div`
    height: 92px;
    display: flex;
    flex-flow: row wrap;
    cursor: default;
    justify-content: space-between;
    background-color: rgba(240, 240, 240, 0.7);
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


function App() {

  return (
    <Provider store={rootStore}>
    <NetworkingProvider>
      <ThemeProvider>
        <BaseStyles>
          <Header data-tauri-drag-region>
            <ProtocolOverview data-tauri-drag-region>
              <ProtocolName data-tauri-drag-region>Counter</ProtocolName>
              <ProtocolLegend data-tauri-drag-region>Protocol</ProtocolLegend>
              {/* <HiroIcon/> */}
            </ProtocolOverview>
            <Chain/>
            {/* <HiroIcon/> */}
          </Header>
          <Box>
            {/* <UnderlineNav aria-label="Main">
              <UnderlineNav.Link href="#state" style={{ width: 112 }} selected>
                <StyledOcticon sx={{ marginRight: 2 }} size={16} icon={VersionsIcon} />
                <Text>State</Text>
              </UnderlineNav.Link>
              <UnderlineNav.Link href="#events" style={{ width: 112 }}>
                <StyledOcticon sx={{ marginRight: 2 }} size={16} icon={ZapIcon} />
                <Text>Lambdas</Text>
              </UnderlineNav.Link>
              <UnderlineNav.Link href="#events" style={{ width: 124 }}>
                <StyledOcticon sx={{ marginRight: 2 }} size={16} icon={DatabaseIcon} />
                <Text>Feeds</Text>
              </UnderlineNav.Link>
              <UnderlineNav.Link href="#settings" style={{ width: 112 }}>
                <StyledOcticon sx={{ marginRight: 2 }} size={16} icon={TerminalIcon} />
                <Text>Extensions</Text>
              </UnderlineNav.Link>
            </UnderlineNav> */}
          </Box>
          <StateExplorer></StateExplorer>
        </BaseStyles>
      </ThemeProvider>
    </NetworkingProvider>
    </Provider>
  );
}

export default App
