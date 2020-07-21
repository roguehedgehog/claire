resource "aws_security_group" "locked_down" {
  name        = "CLAIRE Locked Down"
  description = "Instances assigned to this group cannot communicate on the network"
}
