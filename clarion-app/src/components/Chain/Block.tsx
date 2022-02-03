
import styled from "styled-components";

export const Container = styled.div`
width: 64px;
height: 24px;
border-radius: 4px;
// border: 1px ${(props: { isKnown: boolean }) => props.isKnown ? "solid #AAAAAA" : "dashed #AAAAAA"};
border: 1px ${(props: { isKnown: boolean }) => props.isKnown ? "solid rgba(9, 105, 218, 0.2)" : "dashed rgba(9, 105, 218, 0.2)"};
padding: 2px;
`

// color = "rgb(9, 105, 218)"

export const Inner = styled.div`
// background-color: ${(props: { isKnown: boolean }) => props.isKnown ? "rgba(0, 0, 0, 0.20)" : "rgba(0, 0, 0, 0.1)"};
background-color: ${(props: { isKnown: boolean }) => props.isKnown ? "rgba(9, 105, 218, 0.15)" : "rgba(0, 0, 0, 0.05)"};
// color: ${(props: { isKnown: boolean }) => props.isKnown ? "#FFFFFF" : "#AAAAAA"};
color: ${(props: { isKnown: boolean }) => props.isKnown ? "rgba(9, 105, 218)" : "#AAAAAA"};
border-radius: 2px;
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

const Block = (props: { blockHeight: number, isKnown: boolean }) => {

    return (
        <Container isKnown={props.isKnown}>
            <Inner isKnown={props.isKnown}>
                {props.blockHeight}
            </Inner>
        </Container>
    );
};

export { Block };