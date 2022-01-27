import React from 'react'
import './App.css';
import HiroIcon from './hiro.svg'
import StateExplorer from './pages/StateExplorer'
import {ThemeProvider, BaseStyles, Box, Text, UnderlineNav, StyledOcticon} from '@primer/react'
import {VersionsIcon, DatabaseIcon, TerminalIcon, ZapIcon } from '@primer/octicons-react'
import {NetworkingProvider} from './components/NetworkingProvider';
import { Provider } from 'react-redux'
import { rootStore } from './stores/root'

function App() {

  return (
    <Provider store={rootStore}>
    <NetworkingProvider>
      <ThemeProvider>
        <BaseStyles>
          <Box data-tauri-drag-region style={{ width: '100%', display: 'flex', alignItems: 'center', justifyContent: 'center', height: 48 }}>
            {/* <HiroIcon/> */}
          </Box>
          <Box>
            <UnderlineNav aria-label="Main">
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
            </UnderlineNav>
          </Box>
          <StateExplorer></StateExplorer>
        </BaseStyles>
      </ThemeProvider>
    </NetworkingProvider>
    </Provider>
  );
}

export default App
