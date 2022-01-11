import React from 'react'
import './App.css'
import HiroIcon from './hiro.svg'
import Data from './Data'
import {ThemeProvider, BaseStyles, Box, Text} from '@primer/react'
import {Heading, UnderlineNav, StyledOcticon} from '@primer/react'
import {VersionsIcon, DatabaseIcon, GearIcon, BroadcastIcon } from '@primer/octicons-react'
import {SocketProvider} from './components/SocketProvider';
import {BlockHeader} from './components/BlockHeader';

function App() {
  return (
    <SocketProvider>
      <ThemeProvider>
        <BaseStyles>
          <Box data-tauri-drag-region style={{ width: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', height: 48 }}>
            {/* <HiroIcon/> */}
          </Box>
          <Box>
            <UnderlineNav aria-label="Main">
              <UnderlineNav.Link href="#data" style={{ width: 112 }}>
                <StyledOcticon sx={{ marginRight: 2 }} size={16} icon={VersionsIcon} />
                <Text>State</Text>
              </UnderlineNav.Link>
              <UnderlineNav.Link href="#events" style={{ width: 112 }}>
                <StyledOcticon sx={{ marginRight: 2 }} size={16} icon={BroadcastIcon} />
                <Text>Lambdas</Text>
              </UnderlineNav.Link>
              <UnderlineNav.Link href="#events" style={{ width: 124 }}>
                <StyledOcticon sx={{ marginRight: 2 }} size={16} icon={DatabaseIcon} />
                <Text>Collections</Text>
              </UnderlineNav.Link>
              <UnderlineNav.Link href="#settings" style={{ width: 112 }} selected>
                <StyledOcticon sx={{ marginRight: 2 }} size={16} icon={GearIcon} />
                <Text>Settings</Text>
              </UnderlineNav.Link>
            </UnderlineNav>
          </Box>
          <BlockHeader/>
          <Data></Data>
        </BaseStyles>
      </ThemeProvider>
    </SocketProvider>
  );
}

export default App
