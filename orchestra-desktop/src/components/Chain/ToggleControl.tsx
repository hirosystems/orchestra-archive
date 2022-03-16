
import styled from "styled-components";
import { useRootSelector, useRootDispatch } from "../../hooks/useRootSelector";
import { toggleMining, selectNetworkPaused, buildNextRequest } from "../../states/NetworkingState";
import { MouseEvent } from 'react';

export const Container = styled.div`
    width: 150px;
    height: 48px;
    background-color: ${(props: { color: string }) => props.color};
    color: white;
    border-radius: 24px;

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

const ToggleControl = () => {
    let dispatch = useRootDispatch();
    const networkPaused = useRootSelector(selectNetworkPaused);

    function handleToggle(event: TouchEvent | MouseEvent) {
        event.preventDefault();
        dispatch(toggleMining())
        dispatch(buildNextRequest(1));
    }

    let text = "Pause Mining";
    let color = "rgb(9, 105, 218)";
    if (networkPaused) {
        text = "Resume Mining";
        color = "#6E57FF";
    }

    return (
        <Container color={color}>
            <Inner onClick={handleToggle}>
                {text}
            </Inner>
        </Container>
    );
};

export { ToggleControl };