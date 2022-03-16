
import styled from "styled-components";

export const Container = styled.div`
    width: 150px;
    height: 48px;    
`

export const Inner = styled.div`
    text-transform: uppercase;
    font-size: 12px;
    -webkit-user-select: none;      
    -moz-user-select: none;
    -ms-user-select: none;
    user-select: none;
    cursor: default;
    height: 100%;
    width: 100%;
    text-align: center;
    justify-content: center;
    align-items: center;
    display: flex;
    font-weight: 600;
`

const ChainInfo = () => {
    return (
        <Container>
        </Container>
    );
};

export { ChainInfo };