
import styled from "styled-components";

export const Container = styled.div`
    width: 48px;
    height: 48px;
    background-color: rgba(255, 255, 255, 0.10);
    border-radius: 8px;
`

export const Inner = styled.div`
    color: rgba(255, 255, 255, 0.20);
    text-transform: uppercase;
    font-size: 10px;
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
`

const UnknownBlock = (props: { blockHeight: number }) => {
    return (
        <Container>
            <Inner>
            </Inner>
        </Container>
    );
};

export { UnknownBlock };