select
count(*),
designs.tagging_tier
from tags,
 design_tags, 
 designs 
 where tags.id = design_tags.tag_id 
 and designs.id = design_tags.design_id 
 group by designs.tagging_tier
