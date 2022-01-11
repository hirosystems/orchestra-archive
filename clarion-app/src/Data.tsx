import React from 'react';
import './App.css';
import {Box} from '@primer/react'
import {BranchName, SideNav, Text, FilteredSearch, ButtonGroup, Button, ButtonPrimary, CounterLabel, Dropdown, TextInput} from '@primer/react'
import {SearchIcon} from '@primer/octicons-react'

function Data() {
  return (
      <Box display="flex">
        <Box flexGrow={1} p={3}>
          <Box color="fg.muted" bg="canvas.subtle" p={3}>
            Hello
          </Box>
          <Box borderColor="white" borderWidth={20} borderStyle="solid"></Box>
          <Box color="fg.muted" bg="canvas.subtle" p={3}>
            Hello
          </Box>
        </Box>
        <Box p={3}>
          <SideNav bordered sx={{ maxWidth: 360 }}>
            <SideNav.Link href="#url">
            <FilteredSearch>
              <Dropdown>
                <Dropdown.Button>Filter</Dropdown.Button>
                <Dropdown.Menu direction="sw">
                  <Dropdown.Item>Item 1</Dropdown.Item>
                  <Dropdown.Item>Item 2</Dropdown.Item>
                  <Dropdown.Item>Item 3</Dropdown.Item>
                </Dropdown.Menu>
              </Dropdown>
              <TextInput icon={SearchIcon} />
            </FilteredSearch>
            </SideNav.Link>
            <SideNav.Link href="#url" selected>
              <Text>counter.clar</Text>
            </SideNav.Link>
            <SideNav bordered variant="lightweight" sx={{ py: 3, pl: 6, backgroundColor: "sidenav.selectedBg" }}>
              <SideNav.Link href="#url" selected>
                <BranchName>var</BranchName>
                <Text> counter-id</Text>
              </SideNav.Link>
              <SideNav.Link href="#url">
                <BranchName>map</BranchName>
                <Text> counter-values</Text>
              </SideNav.Link>
              <SideNav.Link href="#url">
                <BranchName>nft</BranchName>
                <Text> counter-nft</Text>
              </SideNav.Link>
            </SideNav>
            <SideNav.Link href="#url" selected>
              <Text>meta-counter.clar</Text>
            </SideNav.Link>
            <SideNav bordered variant="lightweight" sx={{ py: 3, pl: 6, backgroundColor: "sidenav.selectedBg" }}>
              <SideNav.Link href="#url">
                <BranchName>var</BranchName>
                <Text> counter-id</Text>
              </SideNav.Link>
              <SideNav.Link href="#url">
                <BranchName>map</BranchName>
                <Text> counter-values</Text>
                <CounterLabel scheme="secondary">13</CounterLabel>
              </SideNav.Link>
              <SideNav.Link href="#url">
                <BranchName>nft</BranchName>
                <Text> counter-nft</Text>
              </SideNav.Link>
            </SideNav>
          </SideNav>
        </Box>
      </Box>
  );
}

export default Data;
