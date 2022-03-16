import { Text } from '@primer/react'
import styled from "styled-components"

export const Container = styled.div`
    color: white;
    font-size: 18px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 24px;
    cursor: default;
`

const Label = (props: { name: String }) => {

    return (
        <Container>
            <Text> {props.name}</Text>
        </Container>
    );
};

export { Label };