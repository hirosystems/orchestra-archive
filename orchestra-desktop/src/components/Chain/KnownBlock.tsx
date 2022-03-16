
import styled from "styled-components";

export const Container = styled.div`
    width: 64px;
    height: 64px;
    background-color: rgba(255, 255, 255, 0.5);
    border-radius: 8px;
`

export const Inner = styled.div`
    color: #454545;
    text-transform: uppercase;
    font-size: 20px;
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

const KnownBlock = (props: { blockHeight: number }) => {
    return (
        <Container>
            <Inner>
                {props.blockHeight}
            </Inner>
        </Container>
    );
};

export { KnownBlock };