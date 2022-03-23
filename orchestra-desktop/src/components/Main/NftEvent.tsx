import { Timeline, StyledOcticon } from '@primer/react'
import { PaperAirplaneIcon, RocketIcon, FlameIcon } from '@primer/octicons-react'
import styled from "styled-components"
import { NftBurnEvent, NftMintEvent, NftTransferEvent } from '../../states/NetworkingState';

export const Container = styled.div`
    color: rgba(255, 255, 255, 0.8);;
    font-size: 24px;
    font-weight: 600;
    letter-space: 0.03em;
    margin-top: 24px;
    cursor: default;
`

const NftEvent = (props: { event: NftMintEvent | NftTransferEvent | NftBurnEvent }) => {
    let icon = RocketIcon;
    let color = 'success.emphasis';
    let label = '';
    if ('Burn' in props.event) {
        icon = FlameIcon;
        color = 'danger.emphasis';
        label = `Token ${props.event.Burn.asset_identifier} was burnt by ${props.event.Burn.sender}`
    } else if ('Transfer' in props.event) {
        icon = PaperAirplaneIcon;
        color = 'accent.emphasis';
        label = `Token ${props.event.Transfer.asset_identifier} was transfered from ${props.event.Transfer.sender} to ${props.event.Transfer.recipient}`
    } else if ('Mint' in props.event) {
        label = `Token ${props.event.Mint.asset_identifier} was minted for by ${props.event.Mint.recipient}`
    }

    return (
        <Timeline.Item>
            <Timeline.Badge sx={{ bg: color }}>
                <StyledOcticon icon={icon} sx={{ color: 'fg.onEmphasis' }} />
            </Timeline.Badge>
            <Timeline.Body sx={{ color: 'fg.onEmphasis' }}>{label}</Timeline.Body>
        </Timeline.Item>
    );
};

export { NftEvent };