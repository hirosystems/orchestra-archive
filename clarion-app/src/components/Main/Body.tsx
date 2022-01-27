

import { Text, Timeline, StyledOcticon, Link } from '@primer/react'
import {FlameIcon } from '@primer/octicons-react'
import styled from "styled-components"
import { Title, Subtitle, Label } from '.';
import { StateExplorerStateUpdateWatchData } from '../../states/NetworkingState';
import { MapTable } from './MapTable';

export const Container = styled.div`
`

const Body = (props: { field?: StateExplorerStateUpdateWatchData }) => {

    if (props.field === undefined) {
        return (<div>
            <Title name="Empty"/>
        </div>)
    }
 
    let title = props.field.field_name;

    let subtitle = "";
    let value = undefined;
    if ("Var" in props.field.field_values) {
        subtitle = `Data variable of type ${props.field.field_values.Var.value_type}`;
        value = <Label name={props.field.field_values.Var.value}/>;
    } else if ("Map" in props.field.field_values) {
        let entriesCount = props.field.field_values.Map.entries.length;
        let formattedCount = entriesCount === 0 ? "empty" : `${entriesCount} entries`;
        subtitle = `Data map: ${formattedCount}`;
        let entries = [];
        for (let entry of props.field.field_values.Map.entries) {
            entries.push(entry[0]);
        }
        value = <MapTable keyType={props.field.field_values.Map.key_type} valueType={props.field.field_values.Map.value_type} entries={entries}/>
    } else if ("Nft" in props.field.field_values) {
        subtitle = `Non Fungible Token map tracking assets of type ${props.field.field_values.Nft.token_type}`;
    } else if ("Ft" in props.field.field_values) {
        subtitle = "Fungible token";
    }  

    return (
        <Container>
            <Title name={title}/>
            <Subtitle name={subtitle}/>
            {value}

            <Label name="Last events"/>
            <Timeline>
                <Timeline.Item>
                    <Timeline.Badge sx={{bg: 'danger.emphasis'}}>
                        <StyledOcticon icon={FlameIcon} sx={{color: 'fg.onEmphasis'}} />
                    </Timeline.Badge>
                    <Timeline.Body>
                    <Link href="#" sx={{fontWeight: 'bold',  color: 'fg.default', mr: 1}} muted>
                    ST1PQHQKV0RJXZFY1DGX8MNSNYVE3VGZJSRTPGZGM
                    </Link>
                    burnt token <Link href="#" sx={{fontWeight: 'bold', color: 'fg.default', mr: 1}} muted>
                        hot potato
                    </Link>
                    <Link href="#" color="fg.muted" muted>
                        Just now
                    </Link>
                    </Timeline.Body>
                </Timeline.Item>
            </Timeline>
        </Container>
    );
};

export { Body };