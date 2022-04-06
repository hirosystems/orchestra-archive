import styled from "styled-components";

export const Container = styled.div`
    width: 48px;
    height: 48px;
    background-color: rgba(255, 255, 255, 0.95);
    border-radius: 8px;
    box-shadow: 0px 0px 5px 0px #FED100;

`

export const Inner = styled.div`
    color: #6B6B6B;
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
    border: 3px solid #FED100;
    border-radius: 8px;
`

const TipBlock = (props: { blockHeight: number }) => {
    return (
        <Container>
            <Inner>
                {props.blockHeight}
            </Inner>
        </Container>
    );
};

export { TipBlock };