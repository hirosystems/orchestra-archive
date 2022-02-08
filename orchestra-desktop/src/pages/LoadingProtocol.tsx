import '../App.css';
import styled from "styled-components";
import { Spinner } from '@primer/react'

// import Clarinet from './clarinet.svg';

const Container = styled.div`
    padding: 10% 0;
    display: flex;
    justify-content: center;
`

const OutterBox = styled.div`
    width: 500px;
    height: 500px;
    border-radius: 24px;
    padding: 24px;
`

const InnerBox = styled.div`
    background-color: white;
    padding: 64px;
    border-radius: 12px;
    height: 100%;
    border: 3px dashed rgba(9, 105, 218, 0.2);
    text-align: center;

`

const Legend = styled.div`
    color: rgb(100, 100, 100);
    font-size: 18px;
    font-weight: 600;
    letter-space: 0.03em;
    cursor: default;
`

function LoadingProtocol() {
    return (
        <Container data-tauri-drag-region>
            <OutterBox>
                <InnerBox>
                    <Spinner size="large" />
                    <Legend>Loading Protocol</Legend> 
                </InnerBox>
            </OutterBox>
        </Container>
  );
}

export default LoadingProtocol;
