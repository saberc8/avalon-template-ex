import { Metadata } from "next";
import { Box, Grid, GridItem } from "@chakra-ui/react";
import { WorkplaceWelcome } from "@/components/dashboard/Welcome";
import { WorkplaceQuickOperation } from "@/components/dashboard/QuickOperation";

export const metadata: Metadata = {
  title: "工作台 - Next Admin",
};

export default function WorkplacePage() {
  return (
    <Box className="gi_page">
      <Grid
        templateColumns={{ base: "1fr", md: "minmax(0, 1fr) 280px" }}
        gap={4}
      >
        <GridItem>
          <Box bg="white" borderRadius="md" boxShadow="sm" p={4}>
            <WorkplaceWelcome />
          </Box>
        </GridItem>
        <GridItem>
          <WorkplaceQuickOperation />
        </GridItem>
      </Grid>
    </Box>
  );
}
