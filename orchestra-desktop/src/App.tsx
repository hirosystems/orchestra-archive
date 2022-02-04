import React from 'react'
import './App.css';
import { ThemeProvider, BaseStyles } from '@primer/react'
import { NetworkingProvider } from './components/NetworkingProvider';
import { Provider } from 'react-redux'
import { rootStore } from './stores/root'
import Content from './Content';


function App() {
  return (
    <Provider store={rootStore}>
      <NetworkingProvider>
        <ThemeProvider>
          <BaseStyles>
            <Content/>
          </BaseStyles>
        </ThemeProvider>
      </NetworkingProvider>
    </Provider>
  );
}

export default App
