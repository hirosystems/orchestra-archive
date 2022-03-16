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
    background-color: clear;
    padding: 128px;
    border-radius: 12px;
    height: 100%;
    border: 2px dashed rgba(255, 255, 255, 0.2);
    text-align: center;
`

const Legend = styled.div`
    color: rgba(255, 255, 255, 0.6);
    font-size: 18px;
    font-weight: 600;
    letter-space: 0.03em;
    cursor: default;
    margin-top: 20px;
`

function LoadingProtocol() {
    return (
        <Container data-tauri-drag-region>
            <OutterBox>
                <InnerBox>
                    <Spinner size="large" sx={{ color: "white" }} />
                    <Legend>Loading Protocol</Legend>
                </InnerBox>
            </OutterBox>
        </Container>
    );
}

export default LoadingProtocol;
