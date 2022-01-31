
import styled from "styled-components";

export const Container = styled.div`
width: 64px;
height: 24px;
border-radius: 4px;
border: 1px dashed #D8D8D8;
padding: 2px;
`

export const Inner = styled.div`
background-color: ${(props: { isKnown: boolean }) => props.isKnown ? "#D8D8D8" : "clear"};
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
        <Container>
            <Inner isKnown={props.isKnown}>
                {props.blockHeight}
            </Inner>
        </Container>
    );
};

export { Block };