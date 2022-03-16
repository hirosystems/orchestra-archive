import '../App.css';
import styled from "styled-components";

// import Clarinet from './clarinet.svg';

const Container = styled.div`
    padding: 10% 0;
    display: flex;
    justify-content: center;
`

const DragBox = styled.div`
    width: 500px;
    height: 500px;
    border-radius: 24px;
    padding: 24px;
`

const InnerDrag = styled.div`
    background-color: clear;
    padding: 64px;
    border-radius: 12px;
    height: 100%;
    border: 2px dashed rgba(255, 255, 255, 0.2);
    text-align: center;

`

const ClarinetFile = styled.img`
    width: 200px;
    margin-left: 20%;
    margin-right: 20%;
    margin-bottom: 24px;
`

const Legend = styled.div`
    color: rgba(255, 255, 255, 0.8);
    font-size: 18px;
    font-weight: 600;
    letter-space: 0.03em;
    cursor: default;
`

function SelectManifest() {
    return (
        <Container data-tauri-drag-region>
            <DragBox>
                <InnerDrag>
                    <ClarinetFile src={require('../clarinet.svg').default} alt='clarinet' />
                    <Legend>Drag and Drop your <b>Clarinet.toml</b></Legend>
                    <Legend>to get started</Legend>
                </InnerDrag>
            </DragBox>
        </Container>
    );
}

export default SelectManifest;
